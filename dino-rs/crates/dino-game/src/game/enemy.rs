//! Pool A: enemies, bosses, dinosaurs (0x23420), the shared enemy library, the census
//! (0x3242E), target selection (0x32032) and the basic thugs FERRIS / GNEISS / DRIVER
//! (class 2, 0x3383A). Owner: enemy port. See `re/specs/enemy/SPEC.md` sections 2-7.
//!
//! Field layout: the shared header is `Body`. The rest of the 0xE0-byte object is kept as raw
//! bytes (`Ram`) because each class uses those offsets differently (byte here, word there);
//! the named constants below give the class 2 meaning and the offset.

use super::body::Body;
use super::{Game, Handle, Slot, slot_of};

#[path = "boss_common.rs"]
pub mod boss_common;
mod claim;
// The poachers (M6-01 / M6-02): POACHER J. / SKINNER / GUTTER.
mod class13;
mod class13_ai;
mod class13_attack;
#[cfg(test)]
mod class13_tests;
mod class14;
mod class14_ai;
mod class14_entry;
mod class14_parts;
mod class14_states;
#[cfg(test)]
mod class14_tests;
mod class15;
mod class98;
mod class100;
// The ending (M3-23): the explosion clouds (90), the pieces (92, 93) and the flying pieces of the sunset (96).
mod class90;
mod class92;
mod class93;
mod class96;
mod class81;
mod class82;
mod class97;
// The jungle of the opening movie (M1-10, M1-11): the hunters (11), the bird (46), its feather (48), the puff (57).
mod class11;
mod class12;
mod class30;
mod class69;
mod class46;
mod class48;
mod class57;
mod class59;
mod class35;
mod class36;
mod class26;
mod class68;
mod class45;
mod class67;
mod class17;
mod class18;
mod class29;
// Shivat (M6-04, M6-05): the T. rex (24).
mod class24;
mod class24_ai;
// Mack (M6-03): the triceratops (38).
mod class38;
mod class78_80;
// The dinosaur hunter (M6-06).
mod class79;
// LASH T. (M6-07, M6-08): the Lash (3), his chain piece (8) and his mace (62).
mod class3;
mod class62;
// The Butcher (M6-09): the starter (5), the carcass and helpers (51, 53, 58), the fighter (61).
mod class5;
mod class6;
mod class6_ai;
mod class51_58;
mod class52;
mod class61;
mod class61_moves;
mod class61_states;
// Slice (M7-01 .. M7-04): the boss (25, moves in class25_ai.rs) and his boomerang (19).
mod class19;
mod class25;
mod class25_ai;
// Hogg (M6-23 / M6-24): the rider (31) and his motorbike (32).
mod class31;
mod class32;
mod class42;
// The bomb carriers (M6-27): the carrier (64) and its dynamite (77).
mod class63;
mod class64;
mod class77;
// Hazards (M6-29, M6-30): the barrel (65), the rock (66) and the biker (70).
mod class65;
mod class66;
mod class83;
mod class70;
// The opening scene actors (M3-20): the Cadillac (16), the Episode 4 men (27) and leader (43).
mod scene_actors;
mod class20;
mod class20_ai;
mod class20_attack;
mod class20_lib;
mod class10;
mod class10_ai;
mod class10_hold;
mod class10_lib;
mod class10_moves;
mod lib32;
mod scenery;

/// Bytes of an object beyond the shared header, addressed by the original offset.
#[derive(Clone, Debug)]
pub struct Ram(pub [u8; 0xE0]);

impl Default for Ram {
    fn default() -> Self {
        Ram([0; 0xE0])
    }
}

impl Ram {
    pub fn b(&self, o: usize) -> u8 {
        self.0[o]
    }
    pub fn sb(&mut self, o: usize, v: u8) {
        self.0[o] = v;
    }
    pub fn w(&self, o: usize) -> u16 {
        u16::from_be_bytes([self.0[o], self.0[o + 1]])
    }
    pub fn sw(&mut self, o: usize, v: u16) {
        [self.0[o], self.0[o + 1]] = v.to_be_bytes();
    }
    pub fn l(&self, o: usize) -> u32 {
        ((self.w(o) as u32) << 16) | self.w(o + 2) as u32
    }
    pub fn sl(&mut self, o: usize, v: u32) {
        self.sw(o, (v >> 16) as u16);
        self.sw(o + 2, v as u16);
    }
}

/// Offsets used by the pool A code (class 2 meaning; other classes reuse them differently).
pub mod off {
    pub const B5B: usize = 0x5B; // spawn byte (entrance facing)
    pub const W5C: usize = 0x5C; // spawn word
    pub const TARGET: usize = 0x76; // target player
    pub const POISE_T: usize = 0x79; // poise refill timer
    pub const WAIT: usize = 0x7A; // decision wait / counter-attack delay
    pub const SLOT: usize = 0x7C; // approach slot
    pub const ATTACKING: usize = 0x7D; // 1 in attack state; negative = not counted
    pub const T80: usize = 0x80; // generic timer (byte = high byte of the word)
    pub const B83: usize = 0x83;
    pub const W84: usize = 0x84; // name id * 4
    pub const STUCK_T: usize = 0x86; // stuck check timer
    pub const B87: usize = 0x87; // guard flag
    pub const DEST_X: usize = 0x88;
    pub const DEST_Z: usize = 0x8A;
    pub const LAST_X: usize = 0x8C;
    pub const LAST_Z: usize = 0x8E;
    pub const B90: usize = 0x90; // jump attack phase
    pub const OBSTACLE: usize = 0x94;
    pub const DROP: usize = 0x96; // item drop row
    pub const WA0: usize = 0xA0;
    pub const WA2: usize = 0xA2;
    pub const TAUNTED: usize = 0xA4;
    pub const REPOS: usize = 0xA5;
    pub const DETOUR_T: usize = 0xA6;
    pub const DETOUR: usize = 0xA7;
    pub const COMBO: usize = 0xA8;
    pub const DASH_T: usize = 0xA9;
    pub const CURSOR: usize = 0xAA;
    pub const DASH_CHK: usize = 0xAC;
    pub const DASH_KIND: usize = 0xAD;
    pub const DASH_X: usize = 0xAE;
    pub const TURNED: usize = 0xB0;
    pub const WB2: usize = 0xB2;
    pub const START_X: usize = 0xB4;
    pub const IDLE_T: usize = 0xB6;
    pub const PAUSE: usize = 0xB8;
    pub const NO_DASH_ATK: usize = 0xB9;
    pub const FADE_T: usize = 0xBA;
    pub const FADE_STEP: usize = 0xBB;
    pub const OBST_T: usize = 0xBC;
}
use off::*;

#[derive(Clone, Debug, Default)]
pub struct Enemy {
    pub b: Body,
    pub m: Ram,
}

/// Class 2 table addresses (`re/specs/enemy/class2_tables.txt`).
mod t2 {
    pub const IDLE_TIMERS: u32 = 0x33B28;
    pub const FADE_PALS: u32 = 0x33CDA;
    pub const AMBUSH: u32 = 0x33FA4;
    pub const PAUSE: u32 = 0x34186;
    pub const WAIT_E: u32 = 0x34210;
    pub const COMBO_CHOICE: u32 = 0x34338;
    pub const COMBOS: u32 = 0x343E8;
    pub const RECOVER_IDX: u32 = 0x345D0;
    pub const RECOVER: u32 = 0x345F0;
    pub const SLOTS: u32 = 0x3524C;
    pub const ATTACK_POS: u32 = 0x352F8;
    pub const SIDE: u32 = 0x3533C;
    pub const ATTACK_CHANCE: u32 = 0x353B6;
    pub const REPOS_TIME: u32 = 0x3546A;
    pub const REPOS_CHOICE: u32 = 0x354B2;
    pub const DASH_ATTACK: u32 = 0x35796;
    pub const WALK_SPEED: u32 = 0x35832;
    pub const RUN_SPEED: u32 = 0x35882;
}

/// `(a + w) as u16 < 2w`: the original's window test (`addi #w; cmpi #2w; bcc`).
#[inline]
pub fn in_win(d: i16, w: u16) -> bool {
    (d as u16).wrapping_add(w) < 2 * w
}

/// Carry of `addi.w #$400` on `d`: true when -0x400 <= d < 0 ("a < b" for nearby values).
#[inline]
pub fn neg_near(d: i16) -> bool {
    (d as u16 as u32) + 0x400 > 0xFFFF
}

impl Game {
    // ------------------------------------------------------------------ pool A update

    /// 0x23420 pool A update: every live slot in slot order, dispatch on +$20.
    pub fn enemies_update(&mut self) {
        // 0xE96A (just before 0x23420): dizzy count of last frame.
        self.spawn.g.dizzy_prev = self.spawn.g.dizzy;
        self.spawn.g.dizzy = 0;
        // 0xE948..0xE95C (between pool E and pool A): the claim lists 0x6A5A / 0x6A8E (class 10).
        self.combat.clear(0x6A5A);
        self.combat.clear(0x6A8E);
        for i in 0..self.enemies.slots.len() {
            if self.enemies.slots[i].b.live == 0 {
                continue;
            }
            let h = self.enemies.handle(i);
            let mut e = std::mem::take(&mut self.enemies.slots[i]);
            match e.b.k20 / 4 {
                0 => self.enemy_class0_part(&mut e, h, i),
                2 => self.enemy_class2(&mut e, h),
                10 => self.enemy_class10(&mut e, h),
                11 => self.enemy_class11(&mut e, h),
                12 => self.enemy_class12(&mut e, h),
                13 => self.enemy_class13(&mut e, h),
                14 => self.enemy_class14(&mut e, h),
                15 => self.enemy_class15(&mut e, h),
                16 => self.enemy_class16(&mut e, h),
                17 => self.enemy_class17(&mut e, h),
                3 => self.enemy_class3(&mut e, h, i),
                5 => self.enemy_class5(&mut e, h),
                6 => self.enemy_class6(&mut e, h, i),
                8 => self.enemy_class8(&mut e, h, i),
                18 => self.enemy_class18(&mut e, h, i),
                19 => self.enemy_class19(&mut e, h, i),
                25 => self.enemy_class25(&mut e, h, i),
                20 => self.enemy_class20(&mut e, h),
                24 => self.enemy_class24(&mut e, h, i),
                27 => self.enemy_class27(&mut e, h),
                29 => self.enemy_class29(&mut e, h, i),
                31 => self.enemy_class31(&mut e, h, i),
                32 => self.enemy_class32(&mut e, h, i),
                42 => self.enemy_class42(&mut e, h, i),
                63 => self.enemy_class63(&mut e, h, i),
                64 => self.enemy_class64(&mut e, h, i),
                77 => self.enemy_class77(&mut e, h, i),
                30 => self.enemy_class30(&mut e, h),
                34 => self.enemy_class34(&mut e, h),
                38 => self.enemy_class38(&mut e, h, i),
                41 => self.enemy_class41(&mut e, h),
                43 => self.enemy_class43(&mut e, h),
                46 => self.enemy_class46(&mut e, h),
                48 => self.enemy_class48(&mut e, h),
                49 => self.enemy_class49(&mut e, h),
                50 => self.enemy_class50(&mut e, h),
                51 => self.enemy_class51(&mut e, h),
                52 => self.enemy_class52(&mut e, h, i),
                53 => self.enemy_class53(&mut e, h),
                54 => self.enemy_class54(&mut e, h),
                55 => self.enemy_class55(&mut e, h),
                56 => self.enemy_class56(&mut e, h),
                57 => self.enemy_class57(&mut e, h),
                26 => self.enemy_class26(&mut e, h, i),
                68 => self.enemy_class68(&mut e, h, i),
                36 => self.enemy_class36(&mut e, h, i),
                35 => self.enemy_class35(&mut e, h, i),
                45 => self.enemy_class45(&mut e, h, i),
                67 => self.enemy_class67(&mut e, h, i),
                59 => self.enemy_class59(&mut e, h, i),
                58 => self.enemy_class58(&mut e, h),
                60 => self.enemy_class60(&mut e, h),
                61 => self.enemy_class61(&mut e, h, i),
                62 => self.enemy_class62(&mut e, h, i),
                65 => self.enemy_class65(&mut e, h, i),
                66 => self.enemy_class66(&mut e, h, i),
                83 => self.enemy_class83(&mut e, h, i),
                78 => self.enemy_class78(&mut e, h, i),
                79 => self.enemy_class79(&mut e, h, i),
                80 => self.enemy_class80(&mut e, h, i),
                69 => self.enemy_class69(&mut e, h),
                70 => self.enemy_class70(&mut e, h, i),
                81 => self.enemy_class81(&mut e, h),
                82 => self.enemy_class82(&mut e, h, i),
                90 => self.enemy_class90(&mut e, h),
                92 => self.enemy_class92(&mut e, h),
                93 => self.enemy_class93(&mut e, h),
                96 => self.enemy_class96(&mut e, h),
                97 => self.enemy_class97(&mut e, h),
                98 => self.enemy_class98(&mut e, h),
                100 => self.enemy_class100(&mut e, h),
                _ => self.enemy_class_unported(&mut e, h, i),
            }
            self.enemies.slots[i] = e;
        }
        self.enemy_ph_hide_sprites();
    }

    /// Classes not ported yet: a placeholder when the registry has the class (`placeholder.rs`),
    /// else freed at once so waves and thresholds keep working.
    pub fn enemy_class_unported(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        if self.trace_hold(h, &e.b) {
            return; // harness: kept as the snapshot had it
        }
        if super::placeholder::spec_of(e.b.k20 / 4).is_some() {
            return self.enemy_ph_update(e, h, i);
        }
        eprintln!("pool A class {:#x} not ported (freed)", e.b.k20 / 4);
        self.enemy_free_pool_a(e, i);
    }

    pub fn enemy_index(&self, h: Handle) -> usize {
        match slot_of(h) {
            Some(Slot::Enemy(i)) => i,
            _ => panic!("not a pool A handle {h:#06x}"),
        }
    }

    /// 0x4C0A: push one monster into the hostile hit list (0x6B64 / 0x6B66).
    /// Drawing remains the caller's 0x49CA / 0x49C0 call, in the original order.
    pub fn enemy_monster_register(&mut self, h: Handle) {
        self.combat.push(0x6B64, h);
    }

    /// 0x533FA: the hunter pokes its target without changing HP, presence or RNG.
    pub fn enemy_hunter_poke(&mut self, hunter: &Enemy, h: Handle) {
        let target = hunter.m.w(TARGET);
        let i = self.enemy_index(target);
        let e = &mut self.enemies.slots[i];
        e.b.set_state(0x0202_0000);
        e.m.sb(WAIT, 0);
        e.b.attacker = h;
        e.b.who = 0xFF;
        e.b.hitstop = 8;
    }

    /// 0x4F450 (Hopper), 0x493EC (Shivat), 0x53900 (Mack): start colour flashing.
    /// Hopper applies its first palette and consumes one toggle in this same call.
    pub fn enemy_monster_flash_start(&mut self, e: &mut Enemy) {
        match e.b.k20 / 4 {
            29 => {
                e.m.sb(0xA7, 0x10);
                e.m.sb(0xA5, 8);
                e.m.sb(0xA6, e.m.b(0xA4));
                self.enemy_monster_flash_colour(e, 0);
                e.m.sb(0xA7, e.m.b(0xA7).wrapping_sub(1));
            }
            24 => {
                e.m.sb(0xA7, 0xF);
                e.m.sb(0xA5, 8);
                e.m.sb(0xA6, e.m.b(0xA4));
            }
            38 => {
                e.m.sb(0xA5, 0xF);
                e.m.sb(0xA6, 8);
                e.m.sb(0xA7, e.m.b(0xA4));
            }
            _ => unreachable!("monster flash class"),
        }
    }

    /// 0x4F468 / 0x49400 / 0x53914: a byte countdown underflows before the next toggle.
    /// Returns the original carry flag (true only on the last toggle).
    pub fn enemy_monster_flash_tick(&mut self, e: &mut Enemy) -> bool {
        let (timer, count, colour, table) = match e.b.k20 / 4 {
            29 => (0xA5, 0xA7, 0xA6, 0x4F4C8),
            24 => (0xA5, 0xA7, 0xA6, 0x49448),
            38 => (0xA6, 0xA5, 0xA7, 0x53956),
            _ => unreachable!("monster flash class"),
        };
        let old = e.m.b(timer);
        e.m.sb(timer, old.wrapping_sub(1));
        if old != 0 { return false; }
        e.m.sb(timer, self.assets.enemy.t.u8(table, e.m.b(count) as u32));
        let next = e.m.b(colour).wrapping_add(1) & 1;
        e.m.sb(colour, next);
        self.enemy_monster_flash_colour(e, next);
        let left = e.m.b(count).wrapping_sub(1);
        e.m.sb(count, left);
        (left as i8) < 0
    }

    /// 0x4F488 / 0x49420 / 0x53934: stage-dependent palette pair and saved colour.
    fn enemy_monster_flash_colour(&self, e: &mut Enemy, colour: u8) {
        let (table, stage_offset, save) = match e.b.k20 / 4 {
            29 => (0x4F4DC, match self.w.stage { 1 => 2, 4 => 4, 7 => 6, _ => 0 }, Some(0xBB)),
            24 => (0x4945C, if self.w.stage == 4 { 2 } else { 0 }, Some(0xBD)),
            38 => (0x5396A, if self.w.stage == 4 { 2 } else { 0 }, None),
            _ => unreachable!("monster flash class"),
        };
        let palette = self.assets.enemy.t.u8(table, stage_offset + colour as u32);
        e.b.palette_override = palette;
        if let Some(offset) = save { e.m.sb(offset, palette); }
    }

    /// 0x9667E: allocate pool C class 30, the sleeping dinosaur's Z mark. Only the
    /// integer positions are copied; stale fractions and the low variant byte survive.
    /// The class handler 0x966B4 belongs to the pool C effects port.
    pub fn enemy_monster_spawn_z(&mut self, e: &Enemy, h: Handle, variant: u8) {
        let Some(i) = self.effects.alloc() else { return };
        let b = &mut self.effects.slots[i].b;
        b.live = 1;
        b.k20 = 0x78;
        b.set_xi_keep(e.b.xi());
        b.set_yi_keep(e.b.yi());
        b.set_zi_keep(e.b.zi());
        b.attacker = 0xFFFF; // high word of the sign-extended owner long at +$68
        b.hp_bar = h; // low word of that long, +$6A
        b.hp = (e.b.state() >> 16) as u16;
        b.hp_prev = e.b.state() as u16;
        b.var26 = ((variant as u16) << 8) | (b.var26 & 0xFF);
    }

    /// 0x48FC pool A free: clears +$00..+$27 and the bytes +$59 +$63 +$7D +$87 +$96.
    pub fn enemy_free_pool_a(&mut self, e: &mut Enemy, i: usize) {
        // HT1: +$84 and +$5C deliberately survive free and slot reuse.
        super::spawn::clear_header(&mut e.b);
        e.b.hitstop = 0;
        e.b.b63 = 0;
        e.m.sb(ATTACKING, 0);
        e.m.sb(B87, 0);
        e.m.sb(DROP, 0);
        self.enemies.release(i);
    }

    /// Writes the spawn record's +$5C / +$5B / +$96 into the new object (any pool).
    pub fn enemy_set_spawn_extras(&mut self, h: Handle, w5c: u16, b5b: u8, b96: u8) {
        let m = match slot_of(h) {
            Some(Slot::Enemy(i)) => &mut self.enemies.slots[i].m,
            Some(Slot::Prop(i)) => &mut self.props.slots[i].m,
            Some(Slot::Item(i)) => &mut self.items.slots[i].m,
            _ => return,
        };
        m.sw(W5C, w5c);
        m.sb(B5B, b5b);
        m.sb(DROP, b96);
    }

    // ------------------------------------------------------------------ library

    /// 0x49CA for an object taken out of its pool: on-screen flag (0x16FA) + draw list.
    pub fn enemy_draw_check(&mut self, b: &mut Body, h: Handle) {
        let (cx, cy) = if self.w.vertical_scene != 0 { (0, 0) } else { (self.cam.s2.cam_x, self.cam.s2.cam_y) };
        let dx = b.xi().wrapping_sub(cx as i16).wrapping_add(0x100) as u16;
        let dy = b.yi().wrapping_add(b.zi()).wrapping_sub(cy as i16).wrapping_add(0x40) as u16;
        b.visible = (dx <= 0x380 && dy <= 0x1C0) as u8;
        if b.visible != 0 {
            self.draw_list.push(((b.zi() as u16 as u32) << 16) | h as u32);
        }
    }

    /// 0x4BEA: enemies the players can hit this frame (list 0x671E / $6726).
    pub fn enemy_push_hurtable(&mut self, h: Handle) {
        self.combat.push(0x671E, h);
    }

    /// 0x17A4 set_speed(id): +$64 = table, ax = az = 0, velocity of the current direction.
    pub fn enemy_set_speed(&self, b: &mut Body, id: u16) {
        let p = self.assets.enemy.speed_ptrs[id as i8 as i16 as usize];
        b.speed_table = p;
        b.ax = 0;
        b.az = 0;
        let (vx, vz) = self.assets.enemy.speed(p, b.dir);
        b.vx = vx;
        b.vz = vz;
    }

    /// 0x1862: move along +$22 with the speed table (Y untouched).
    pub fn enemy_move_dir(&self, b: &mut Body) {
        let (sx, sz) = self.assets.enemy.speed(b.speed_table, b.dir);
        b.vx = sx.wrapping_add(b.ax);
        b.x = b.x.wrapping_add((b.vx as i32) << 8);
        b.vz = sz.wrapping_add(b.az);
        b.z = b.z.wrapping_add((b.vz as i32) << 8);
    }

    /// 0x120E.
    pub fn enemy_anim(&self, b: &mut Body, id: u16) {
        b.start_anim(&self.assets.sprites, id);
    }

    /// 0x121E: returns true when the animation ended (carry).
    pub fn enemy_step(&self, b: &mut Body) -> bool {
        b.tick_anim(&self.assets.sprites)
    }

    /// 0x1380: step backwards; returns the carry.
    pub fn enemy_step_back(&self, b: &mut Body) -> bool {
        b.timer = b.timer.wrapping_sub(1);
        if b.timer != 0 {
            return false;
        }
        let s = &self.assets.sprites;
        let addr = s.frame(b.frame).rom_addr;
        let (prev, carry) = *self.assets.enemy.reverse_frames.get(&addr).unwrap_or_else(|| panic!("no reverse step for frame {addr:#x}"));
        let id = s.frame_at(prev);
        b.load_frame(s, id);
        carry
    }

    /// 0x324D4: walk animation forward when moving toward the facing side, else backwards.
    pub fn enemy_walk_anim(&self, b: &mut Body) {
        self.enemy_walk_anim_c(b);
    }

    /// 0x324D4 with its carry (the animation ended or wrapped).
    pub fn enemy_walk_anim_c(&self, b: &mut Body) -> bool {
        if (b.dir >> 4) != b.facing {
            self.enemy_step(b)
        } else {
            self.enemy_step_back(b)
        }
    }

    /// 0x103A: direction 0..31 from the object to (tx, tz).
    pub fn enemy_angle_to(b: &Body, tx: u16, tz: u16) -> u8 {
        let mut oct: u8 = 0;
        let x = b.xi() as u16;
        let z = b.zi() as u16;
        let (bx, btx) = (x.wrapping_add(0x4000), tx.wrapping_add(0x4000));
        let mut dx = btx.wrapping_sub(bx);
        if btx < bx {
            dx = dx.wrapping_neg();
            oct = 4;
        }
        let (bz, btz) = (z.wrapping_add(0x4000), tz.wrapping_add(0x4000));
        let mut dz = btz.wrapping_sub(bz);
        if btz < bz {
            dz = dz.wrapping_neg();
            oct += 2;
        }
        let (small, large);
        if dz < dx {
            small = dz;
            large = dx;
            if oct == 0 || oct == 4 {
                oct += 1;
            }
        } else {
            small = dx;
            large = dz;
            if oct == 2 || oct == 6 {
                oct += 1;
            }
        }
        let l8 = large >> 3;
        let step = l8.wrapping_add(l8);
        let mut acc = l8;
        let mut k = 0u8;
        if acc < small {
            k = 4;
            for kk in 1..=3u8 {
                // `add.b d4,d2` (0x10B2): only the low byte of the accumulator changes.
                acc = (acc & 0xFF00) | (acc as u8).wrapping_add(step as u8) as u16;
                if acc >= small {
                    k = kk;
                    break;
                }
            }
        }
        const T: [[u8; 5]; 8] = [
            [0x00, 0x01, 0x02, 0x03, 0x04],
            [0x08, 0x07, 0x06, 0x05, 0x04],
            [0x08, 0x09, 0x0A, 0x0B, 0x0C],
            [0x10, 0x0F, 0x0E, 0x0D, 0x0C],
            [0x00, 0x1F, 0x1E, 0x1D, 0x1C],
            [0x18, 0x19, 0x1A, 0x1B, 0x1C],
            [0x18, 0x17, 0x16, 0x15, 0x14],
            [0x10, 0x11, 0x12, 0x13, 0x14],
        ];
        T[oct as usize][k as usize]
    }

    /// 0x191E: |dx| + |dz|.
    pub fn enemy_manhattan(a: &Body, b: &Body) -> u16 {
        let dx = (b.xi().wrapping_sub(a.xi())) as u16;
        let dz = (b.zi().wrapping_sub(a.zi())) as u16;
        let ax = if b.xi() as u16 >= a.xi() as u16 { dx } else { dx.wrapping_neg() };
        let az = if b.zi() as u16 >= a.zi() as u16 { dz } else { dz.wrapping_neg() };
        ax.wrapping_add(az)
    }

    /// 0x324EE: every 30 frames, true when moved less than 8 px in X and Z.
    pub fn enemy_stuck_check(e: &mut Enemy) -> bool {
        let t = e.m.b(STUCK_T).wrapping_sub(1);
        e.m.sb(STUCK_T, t);
        if t != 0 {
            return false;
        }
        e.m.sb(STUCK_T, 0x1E);
        let stuck = in_win(e.b.xi().wrapping_sub(e.m.w(LAST_X) as i16), 8) && in_win(e.b.zi().wrapping_sub(e.m.w(LAST_Z) as i16), 8);
        e.m.sw(LAST_X, e.b.xi() as u16);
        e.m.sw(LAST_Z, e.b.zi() as u16);
        stuck
    }

    /// 0x193E: keep the object inside the play area (X camX-0x50..camX+0x1D0, Z band).
    pub fn enemy_clamp_to_playfield(&mut self, e: &mut Enemy, i: usize) {
        if self.spawn.g.car_mode != 0 {
            self.enemy_despawn_if_far(e, i);
            return;
        }
        let b = &mut e.b;
        if self.w.vertical_scene != 0 {
            let d = b.xi().wrapping_sub(0x10);
            if d < 0 {
                b.set_xi_keep(b.xi().wrapping_sub(d));
            } else {
                let d = d.wrapping_sub(0x160);
                if d >= 0 {
                    b.set_xi_keep(b.xi().wrapping_sub(d));
                }
            }
            let d = b.zi();
            if d < 0 {
                b.set_zi_keep(0);
            } else {
                let d = d.wrapping_sub(0x140);
                if d >= 0 {
                    b.set_zi_keep(b.zi().wrapping_sub(d));
                }
            }
            return;
        }
        let d = b.xi().wrapping_sub(self.cam.s2.cam_x as i16).wrapping_add(0x50);
        if d < 0 {
            b.set_xi_keep(b.xi().wrapping_sub(d));
        } else {
            let d = d.wrapping_sub(0x220);
            if d >= 0 {
                b.set_xi_keep(b.xi().wrapping_sub(d));
            }
        }
        let d = b.zi().wrapping_add(b.floor2).wrapping_add(b.floor).wrapping_sub(self.cam.s2.cam_y as i16);
        if d < 0 {
            b.set_zi_keep(b.zi().wrapping_sub(d));
        } else {
            let d = d.wrapping_sub(0x100);
            if d >= 0 {
                b.set_zi_keep(b.zi().wrapping_sub(d));
            }
        }
    }

    /// 0x173C: free next frame (+$04 = 6) when far off screen. In a vertical scene (G.$4DC, 0x1782) the limits are |X| and Z + Y.
    pub fn enemy_despawn_if_far(&mut self, e: &mut Enemy, _i: usize) {
        if self.w.vertical_scene != 0 {
            if (e.b.xi().wrapping_add(0x80) as u16) >= 0x280
                || (e.b.zi().wrapping_add(e.b.yi()).wrapping_add(0x80) as u16) >= 0x200
            {
                e.b.mode = 6;
            }
            return;
        }
        let d = e.b.xi().wrapping_sub(self.cam.s2.cam_x as i16).wrapping_add(0xC0) as u16;
        if d >= 0x300 {
            e.b.mode = 6;
        }
    }

    /// 0x1760: the narrower off-screen test (the barrel). Same vertical limits as 0x173C (0x1782). Sets mode 6 only.
    pub fn enemy_despawn_near(&mut self, e: &mut Enemy) {
        if self.w.vertical_scene != 0 {
            if (e.b.xi().wrapping_add(0x80) as u16) >= 0x280
                || (e.b.zi().wrapping_add(e.b.yi()).wrapping_add(0x80) as u16) >= 0x200
            {
                e.b.mode = 6;
            }
            return;
        }
        let d = e.b.xi().wrapping_sub(self.cam.s2.cam_x as i16).wrapping_add(0x40) as u16;
        if d >= 0x240 {
            e.b.mode = 6;
        }
    }

    /// 0x12CB4 terrain collision of an enemy (floor, walls, street edges, obstacles; see
    /// `terrain.rs`). Obstacle contact (a container in the way) is stored at +$94, bit 7 of
    /// +$50 tells class 2 to attack it.
    pub fn enemy_floor_collision(&mut self, e: &mut Enemy) {
        if let Some(o) = self.terrain_collide(&mut e.b, 0, false) {
            e.m.sw(OBSTACLE, o);
        }
    }

    /// 0x3140C: terrain collision, then stand on the floor.
    pub fn enemy_snap_to_floor(&mut self, e: &mut Enemy) {
        self.enemy_floor_collision(e);
        let f = e.b.floor.wrapping_add(e.b.floor2);
        e.b.y = (e.b.y & 0xFFFF) | ((f as i32) << 16);
        e.b.airborne = 0;
    }

    /// 0x31424: HP from the table by name id and level.
    pub fn enemy_set_hp(&self, b: &mut Body) {
        let hp = self.assets.enemy.hp[b.id as usize][(self.w.level & 31) as usize];
        b.hp_bar = hp.min(100);
        b.hp = hp;
        b.hp_prev = hp;
    }

    /// 0x31E14: boss HP by boss index, players and level.
    pub fn enemy_set_boss_hp(&self, b: &mut Body, idx: usize) {
        let row = &self.assets.enemy.boss_hp[idx][self.w.players_minus_one as usize % 3];
        let hp = row[((self.w.level & 0x1C) >> 2) as usize];
        b.hp_bar = hp.min(100);
        b.hp = hp;
        b.hp_prev = hp;
    }

    // ------------------------------------------------------------------ census, targets

    /// 0x3242E enemy census (first call of the object update).
    pub fn enemy_census(&mut self) {
        let g = &mut self.spawn.g;
        g.census_prev = g.census;
        g.census = [[0; 4]; 3];
        g.alive = g.alive_run;
        g.alive_run = 0;
        for i in 0..self.enemies.slots.len() {
            let e = &self.enemies.slots[i];
            if e.b.live == 0 || (e.m.b(ATTACKING) as i8) < 0 || e.b.mode >= 4 {
                continue;
            }
            self.spawn.g.alive_run = self.spawn.g.alive_run.wrapping_add(1);
            if e.b.mode == 0 {
                continue;
            }
            let t = e.m.w(TARGET);
            let p = if t == Game::player_handle(2) {
                2
            } else if t == Game::player_handle(1) {
                1
            } else {
                0
            };
            let c = &mut self.spawn.g.census[p];
            c[0] = c[0].wrapping_add(1);
            if e.m.b(ATTACKING) != 0 {
                c[1] = c[1].wrapping_add(1);
            }
            if self.players[p].b.xi() as u16 >= e.b.xi() as u16 {
                c[2] = c[2].wrapping_add(1);
            } else {
                c[3] = c[3].wrapping_add(1);
            }
        }
    }

    /// 0x32032: choose the target player for enemy `b`.
    pub fn enemy_pick_target(&self, b: &Body) -> Handle {
        let present: [bool; 3] = std::array::from_fn(|i| self.players[i].b.live != 0);
        if !present.iter().any(|&p| p) {
            return Game::player_handle(0); // 0x32376 with G.$4CD unknown: P1
        }
        let mut score = [0u8; 3];
        let prev = &self.spawn.g.census_prev;
        let mut best = 0;
        let mut m = 0x10u8;
        if present[0] {
            m = prev[0][0];
        }
        for p in 1..3 {
            if present[p] && prev[p][0] <= m {
                best = p;
                m = prev[p][0];
            }
        }
        if m < 3 {
            score[best] = 1;
        } else {
            // Nearest (Manhattan, unsigned compares, ties keep the earlier player): 10.
            let dist = |p: usize| -> u16 {
                if !present[p] {
                    return 0xFFFF;
                }
                let q = &self.players[p].b;
                let dx = (q.xi().wrapping_sub(b.xi())).unsigned_abs();
                let dz = (q.zi().wrapping_sub(b.zi())).unsigned_abs();
                dx.wrapping_add(dz)
            };
            let mut n = 0;
            let mut d0 = dist(0);
            for p in 1..3 {
                let d = dist(p);
                if d < d0 {
                    d0 = d;
                    n = p;
                }
            }
            score[n] = 10;
            // Largest score (BCD long +$100), ties keep the earlier player: +1.
            let mut n = 0;
            let mut v = 0u32;
            for p in 0..3 {
                if present[p] && (p == 0 || self.enemy_player_score(p) > v) {
                    if p == 0 || self.enemy_player_score(p) > v {
                        v = self.enemy_player_score(p);
                        n = p;
                    }
                }
            }
            score[n] = score[n].wrapping_add(1);
            // Largest vitality: +2.
            let mut n = 0;
            let mut v = 0u16;
            for p in 0..3 {
                if present[p] && (p == 0 || self.players[p].b.hp > v) {
                    v = self.players[p].b.hp;
                    n = p;
                }
            }
            score[n] = score[n].wrapping_add(2);
            for p in 0..3 {
                if !present[p] {
                    continue;
                }
                let q = &self.players[p].b;
                let a0 = self.enemy_player_move_a0(p);
                if a0 != 0 && a0 & 0x0F00 == 0x0200 {
                    let lo = a0 as u8;
                    score[p] = score[p].wrapping_add(if (4..10).contains(&lo) { 6 } else { 3 });
                }
                // QUIRK (0x3221C): both branches test facing == 0.
                if b.facing == 0 {
                    score[p] = score[p].wrapping_add(4);
                }
                if q.mode == 2 && q.sub == 0 && q.act == 2 {
                    score[p] = score[p].wrapping_add(3);
                }
            }
        }
        let mut n = 0;
        if score[1] > score[n] {
            n = 1;
        }
        if score[2] > score[n] {
            n = 2;
        }
        Game::player_handle(n)
    }

    // ------------------------------------------------------------------ interface needed from the player port

    /// Player +$100 score (BCD long). TODO(player port): replace with the real field.
    pub fn enemy_player_score(&self, p: usize) -> u32 {
        self.players[p % 3].score
    }

    /// Player +$A0 current move word (0x02xx = attack). TODO(player port): real field.
    pub fn enemy_player_move_a0(&self, _p: usize) -> u16 {
        0
    }

    /// 0x10036: add BCD `v` to player `p`'s score (+$100), saturating at 99999999.
    pub fn enemy_add_score(&mut self, p: u8, v: u32) {
        if (p as i8) < 0 {
            return;
        }
        let i = p as usize & 3;
        let i = if i == 3 { 0 } else { i }; // table 0x2B002 maps index 3 to P1
        self.players[i].score = bcd_add_long(self.players[i].score, v);
        // HUD redraw flags (-$7134 and the player's -$7133 / -$7132 / -$7131).
        self.hud.score_dirty = true;
        self.hud.score_dirty_p[i] = true;
    }

    // ------------------------------------------------------------------ kill-all, dying

    /// 0x326F8: kill-all (G.$50E) turns every living enemy into a dying knockdown.
    pub fn enemy_kill_all_check(&mut self, e: &mut Enemy) {
        let k = self.spawn.g.kill_all;
        if k == 0 || e.b.hp == 0 {
            return;
        }
        e.b.hp = 0;
        e.b.set_state(0x0202_0000);
        e.b.hitstop = 2;
        e.b.stun = 0;
        let killer = self.spawn.g.killer;
        e.b.attacker = killer;
        e.b.who = if killer != 0 { self.body(killer).who } else { 0 };
        if k == 2 {
            e.b.who = 0xFF;
        }
    }

    /// 0x325E6: kill score for name id to the credited player (+$02).
    pub fn enemy_award_kill_score(&mut self, b: &Body) {
        let t = &self.assets.enemy.t;
        let off = t.u8(0x325F8, b.id as u32);
        let v = t.u32(0x10292, off as u32);
        self.enemy_add_score(b.who, v);
    }

    /// 0x3484E: class 2 death: kill score, blink for 120 frames, then free.
    pub fn enemy_dying_basic(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.sub {
            0 => {
                let b = e.b.clone();
                self.enemy_award_kill_score(&b);
                e.m.sw(T80, 0x78);
                e.b.sub += 2;
                self.enemy_dying_blink(e, h);
            }
            _ => self.enemy_dying_blink(e, h),
        }
    }

    pub fn enemy_dying_blink(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.w(T80);
        if t == 0 {
            e.b.mode += 2;
            return;
        }
        let t = t - 1;
        e.m.sw(T80, t);
        if t & 1 != 0 {
            self.enemy_draw_check(&mut e.b, h);
        }
    }

    // ------------------------------------------------------------------ class 2: FERRIS / GNEISS / DRIVER

    /// 0x3383A class 2.
    pub fn enemy_class2(&mut self, e: &mut Enemy, h: Handle) {
        let i = self.enemy_index(h);
        match e.b.mode {
            0 => self.enemy_c2_init(e, h),
            2 => self.enemy_c2_alive(e, h, i),
            4 => self.enemy_dying_basic(e, h),
            _ => self.enemy_free_pool_a(e, i),
        }
    }

    pub fn enemy_variant(e: &Enemy) -> u32 {
        (e.b.var26 >> 8) as u32 & 0xFF
    }

    pub fn enemy_aggr(&self) -> u32 {
        self.rank.aggression as u32 & 0xFFFF
    }

    /// 0x35812.
    pub fn enemy_c2_walk_speed(&self, e: &mut Enemy) {
        let off = (Self::enemy_variant(e) << 3).wrapping_add((self.rank.aggression & 7) as u32 * 2);
        let id = self.assets.enemy.t.u16(t2::WALK_SPEED, off);
        self.enemy_set_speed(&mut e.b, id);
    }

    /// 0x35862.
    pub fn enemy_c2_run_speed(&self, e: &mut Enemy) {
        let off = (Self::enemy_variant(e) << 3).wrapping_add((self.rank.aggression & 7) as u32 * 2);
        let id = self.assets.enemy.t.u16(t2::RUN_SPEED, off);
        self.enemy_set_speed(&mut e.b, id);
    }

    /// 0x3384E init (+$04 = 0).
    pub fn enemy_c2_init(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.sub {
            0 => {
                e.b.sub += 2;
                // 0x939B6: test-mode debug marker only.
                let (id, w84, table) = match Self::enemy_variant(e) {
                    0 => (0x00, 0x00, 0x34892),
                    2 => (0x01, 0x04, 0x34B90),
                    4 => (0x1A, 0x68, 0x34E7C),
                    v => panic!("class 2 variant {v:#x}"),
                };
                e.b.id = id;
                e.m.sw(W84, w84);
                e.b.anim_table = table;
                self.enemy_anim(&mut e.b, 0);
                e.b.box_table = 0x103000;
                e.b.multi_box = false;
                self.enemy_c2_walk_speed(e);
                self.enemy_set_hp(&mut e.b);
                // 0x33926 common init
                e.b.stun = 10;
                for o in [POISE_T, WAIT] {
                    e.m.sb(o, 0);
                }
                e.b.reaction = 0;
                e.m.sb(ATTACKING, 0);
                e.m.sb(B87, 0);
                e.b.b63 = 0;
                e.b.b72 = 0xB4;
                e.m.sb(TAUNTED, 0);
                e.m.sw(WA2, 0);
                e.m.sb(REPOS, 0xFF);
                e.m.sb(DASH_T, 0x14);
                e.m.sb(DASH_CHK, 0x5A);
                e.m.sb(FADE_T, 0);
                e.m.sb(FADE_STEP, 0);
                e.m.sb(OBST_T, 0);
                let t = self.enemy_pick_target(&e.b);
                e.m.sw(TARGET, t);
                self.enemy_snap_to_floor(e);
                e.m.sb(ATTACKING, 0);
            }
            2 => {
                self.enemy_kill_all_check(e);
                match e.b.var26 & 0xFF {
                    0 => self.enemy_c2_entr_walk(e),
                    2 => self.enemy_c2_entr_dash(e),
                    4 => self.enemy_c2_entr_wait(e),
                    6 => self.enemy_c2_entr_fade(e),
                    8 => self.enemy_c2_entr_jump(e, h),
                    0xA => self.enemy_c2_entr_drop(e),
                    v => panic!("class 2 entrance {v:#x}"),
                }
                self.enemy_draw_check(&mut e.b, h);
                self.enemy_push_hurtable(h);
            }
            _ => {
                // 0x33E66: entrance finished.
                self.enemy_c2_retarget_slot(e);
                self.enemy_c2_reset_ai(e, h);
                self.enemy_draw_check(&mut e.b, h);
            }
        }
    }

    pub fn enemy_face_screen_centre(&self, e: &mut Enemy) {
        let d = (self.cam.s2.cam_x as i16).wrapping_add(0xC0).wrapping_sub(e.m.w(START_X) as i16);
        if d >= 0 {
            e.b.dir = 8;
            e.b.facing = 1;
        } else {
            e.b.dir = 0x18;
            e.b.facing = 0;
        }
    }

    /// 0x33B96: a present player inside the box `ahead` px in front.
    pub fn enemy_player_ahead(&self, e: &Enemy, ahead: i16, half: u16, w: u16) -> bool {
        let px = e.b.xi().wrapping_add(if e.b.facing != 0 { ahead } else { -ahead });
        for p in 0..3 {
            let q = &self.players[p].b;
            if q.live == 0 {
                continue;
            }
            let d = px.wrapping_sub(q.xi()).wrapping_add(half as i16) as u16;
            if d >= w {
                continue;
            }
            let dz = e.b.zi().wrapping_sub(q.zi()).wrapping_add(0xC) as u16;
            if dz < 0x18 {
                return true;
            }
            // The original returns the Z test's carry: a player beside but not inside Z stops
            // the scan only for P1 and P2 when the X test passed (bcc continues).
        }
        false
    }

    /// 0x339B8 entrance 0: walk in.
    pub fn enemy_c2_entr_walk(&mut self, e: &mut Enemy) {
        if e.b.act == 0 {
            e.m.sw(START_X, e.b.xi() as u16);
            self.enemy_anim(&mut e.b, 1);
            self.enemy_c2_walk_speed(e);
            self.enemy_face_screen_centre(e);
            e.b.act += 2;
            return;
        }
        if self.enemy_player_ahead(e, 0x18, 0x18, 0x30) || !in_win(e.b.xi().wrapping_sub(e.m.w(START_X) as i16), 0x80) {
            e.b.sub += 2;
        }
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }

    /// 0x33A42 entrance 2: dash in at 4 px/frame.
    pub fn enemy_c2_entr_dash(&mut self, e: &mut Enemy) {
        if e.b.act == 0 {
            e.m.sw(START_X, e.b.xi() as u16);
            self.enemy_anim(&mut e.b, 6);
            self.enemy_set_speed(&mut e.b, 0x1F);
            self.enemy_face_screen_centre(e);
            e.b.act += 2;
            return;
        }
        let d = e.b.xi().wrapping_sub(e.m.w(START_X) as i16);
        let mut done = false;
        if !in_win(d, 0x40) && self.enemy_player_ahead(e, 0x20, 0x20, 0x40) {
            done = true;
        }
        if !done && !in_win(d, 0x80) {
            done = true;
        }
        if done {
            e.b.sub += 2;
        }
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }

    /// 0x33BE8: a present player within Manhattan distance 0x50 starts the AI.
    pub fn enemy_player_near(&self, e: &mut Enemy) {
        for p in 0..3 {
            let q = &self.players[p].b;
            if q.live != 0 && Self::enemy_manhattan(&e.b, q) < 0x50 {
                e.b.sub += 2;
                return;
            }
        }
    }

    /// 0x33AE2 entrance 4: wait (stand or sit) until a player is near.
    pub fn enemy_c2_entr_wait(&mut self, e: &mut Enemy) {
        match e.b.act {
            0 => {
                e.b.act += 2;
                let r = self.rand() & 0x3E;
                let v = self.assets.enemy.t.u16(t2::IDLE_TIMERS, r as u32);
                e.m.sw(IDLE_T, v);
                let f = e.m.b(B5B);
                if (f as i8) < 0 {
                    self.enemy_anim(&mut e.b, 0xA);
                } else {
                    e.b.facing = f;
                    self.enemy_anim(&mut e.b, 0xB);
                }
            }
            2 => {
                let d = (self.cam.s2.cam_x as i16).wrapping_add(0xC0).wrapping_sub(e.b.xi()).wrapping_add(0xC0) as u16;
                if d < 0x180 {
                    e.b.act += 2;
                }
                self.enemy_player_near(e);
            }
            _ => {
                let t = e.m.w(IDLE_T);
                e.m.sw(IDLE_T, t.wrapping_sub(1));
                if t == 0 {
                    e.b.sub += 2;
                    return;
                }
                self.enemy_player_near(e);
            }
        }
    }

    /// 0x33C32 entrance 6: fade in from a dark doorway.
    pub fn enemy_c2_entr_fade(&mut self, e: &mut Enemy) {
        if e.b.act == 0 {
            e.b.act += 2;
            e.b.palette_override = 0x9D;
            e.b.z = e.b.z.wrapping_add(0x10 << 16);
            e.b.vz = -0x40;
            e.m.sb(FADE_T, 0x14);
            self.enemy_c2_fade_step(e);
            self.enemy_anim(&mut e.b, 0xD);
            return;
        }
        let t = e.m.b(FADE_T).wrapping_sub(1);
        e.m.sb(FADE_T, t);
        if t == 0 {
            if e.m.b(FADE_STEP) == 0xC {
                e.b.sub += 2;
                e.b.palette_override = 0;
                return;
            }
            e.m.sb(FADE_T, 0x14);
            e.m.sb(FADE_STEP, e.m.b(FADE_STEP).wrapping_add(4));
            self.enemy_c2_fade_step(e);
        }
        e.b.motion();
        self.enemy_step(&mut e.b);
    }

    /// 0x33C9E: copy one fade step into OBJ palette 0x1D, keeping each colour's brightness.
    pub fn enemy_c2_fade_step(&mut self, e: &Enemy) {
        let v = Self::enemy_variant(e);
        let t = &self.assets.enemy.t;
        let src = t.u32(t2::FADE_PALS, v * 8 + e.m.b(FADE_STEP) as u32);
        let base = 0x1D * 16; // 0x9143A0 (all three variants use palette 0x1D)
        for k in 0..16 {
            let w = self.pal.words[base + k];
            self.pal.words[base + k] = (w & 0xF000) | t.u16(src, 2 * k as u32);
        }
        self.pal.dirty = true;
    }

    /// 0x33D0A entrance 8: jump in.
    pub fn enemy_c2_entr_jump(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.act {
            0 => {
                e.b.act += 2;
                self.enemy_floor_collision(e);
                e.b.y = e.b.y.wrapping_add(0x20 << 16);
                e.b.airborne = 1;
                e.b.reaction = 4;
                e.b.stun = 10;
                e.b.vx = if e.b.facing != 0 { 0x600 } else { -0x600 };
                e.b.ax = 0;
                e.b.az = 0;
                e.b.vz = 0;
                e.b.vy = 0x300;
                e.b.ay = -0x50;
                self.enemy_anim(&mut e.b, 0xE);
            }
            2 => {
                self.combat.push(0x6AC2, h); // 0x4B58
                e.b.motion();
                if e.b.param_a == 0 {
                    self.enemy_step(&mut e.b);
                    let d = e.b.param_b;
                    if d != 0 {
                        e.b.param_b = 0;
                        e.b.facing = e.b.facing.wrapping_add(d) & 1;
                    }
                }
                self.enemy_floor_collision(e);
                if e.b.airborne == 0 {
                    e.b.airborne = 1;
                    e.b.vy = e.b.vy.wrapping_neg();
                    e.b.ay = e.b.ay.wrapping_add(e.b.ay);
                    e.b.act += 2;
                    self.snd_9796(e.b.xi()); // 0x33DCE
                    self.enemy_step(&mut e.b);
                }
            }
            4 => {
                self.combat.push(0x6AC2, h);
                e.b.motion();
                if e.b.param_a == 0 {
                    self.enemy_step(&mut e.b);
                }
                self.enemy_floor_collision(e);
                if e.b.airborne == 0 {
                    e.m.sb(T80, if e.b.hp == 0 { 0xA } else { 0x14 });
                    self.snd_9796(e.b.xi()); // 0x33E16
                    e.b.act += 2;
                }
            }
            6 => {
                let t = e.m.b(T80);
                e.m.sb(T80, t.wrapping_sub(1));
                if t == 0 {
                    e.b.act += 2;
                    self.enemy_anim(&mut e.b, 0xF);
                } else {
                    self.enemy_step(&mut e.b);
                }
            }
            _ => {
                if self.enemy_step(&mut e.b) {
                    e.b.sub += 2;
                }
            }
        }
    }

    /// 0x33E48 entrance 0xA: drop from 256 px.
    pub fn enemy_c2_entr_drop(&mut self, e: &mut Enemy) {
        self.enemy_floor_collision(e);
        e.b.y = e.b.y.wrapping_add(0x100 << 16);
        e.b.airborne = 1;
        e.b.facing = e.m.b(B5B);
        e.b.sub += 2;
    }

    /// 0x33E76 alive.
    pub fn enemy_c2_alive(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        self.enemy_kill_all_check(e);
        match e.b.sub {
            0 => self.enemy_c2_ai(e, h),
            2 => self.enemy_hit_reaction(e, h),
            6 => self.enemy_grabbed(e, h),
            s => panic!("class 2 alive sub {s:#x}"),
        }
        self.enemy_clamp_to_playfield(e, i);
    }

    /// 0x33EA4 AI frame.
    pub fn enemy_c2_ai(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.b(POISE_T);
        if t != 0 {
            e.m.sb(POISE_T, t - 1);
            if t - 1 == 0 {
                e.b.stun = 10;
            }
        }
        match e.b.act {
            0x0 => self.enemy_c2_decide(e, h),
            0x2 => self.enemy_c2_approach(e, h),
            0x4 => self.enemy_c2_taunt(e),
            0x6 => self.enemy_c2_hover(e, h),
            0x8 => self.enemy_c2_attack(e, h),
            0xA => self.enemy_c2_dash(e, h),
            0xC => self.enemy_c2_fall(e, h),
            0xE => self.enemy_c2_wait_attack(e),
            0x10 => self.enemy_c2_hop(e, h),
            a => panic!("class 2 AI state {a:#x}"),
        }
        self.enemy_floor_collision(e);
        let t = e.m.b(OBST_T).wrapping_sub(1);
        e.m.sb(OBST_T, t);
        if t == 0 {
            e.m.sb(OBST_T, 0xC8);
            if e.b.wall & 0x80 != 0 {
                self.enemy_c2_attack_obstacle(e);
            }
        }
        self.enemy_c2_ai_tail(e, h);
    }

    /// 0x33EE2: the end of the AI frame (also run by `c2_reset_ai`).
    pub fn enemy_c2_ai_tail(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.wall == 0x10 {
            e.b.set_state(0x0200_1000);
        }
        self.enemy_draw_check(&mut e.b, h);
        self.enemy_push_hurtable(h);
    }

    /// 0x33F0C: attack the obstacle at +$94 with combo 7.
    pub fn enemy_c2_attack_obstacle(&mut self, e: &mut Enemy) {
        e.b.set_state(0x0200_0806);
        e.m.sb(COMBO, 7);
        e.m.sw(CURSOR, 0x2A);
        let o = e.m.w(OBSTACLE);
        let ox = if o != 0 { self.body(o).xi() } else { e.b.xi() };
        e.b.facing = (ox.wrapping_sub(e.b.xi()) > 0) as u8;
    }

    /// 0x33F38: back to state 0 (runs the AI tail: the object is registered a second time
    /// when this is called from inside a state handler, as in the original).
    pub fn enemy_c2_reset_ai(&mut self, e: &mut Enemy, h: Handle) {
        e.b.set_state(0x0200_0000);
        self.enemy_anim(&mut e.b, 1);
        if e.m.b(WAIT) == 0 {
            e.m.sb(WAIT, 2);
        }
        e.m.sb(TAUNTED, 0);
        self.enemy_c2_walk_speed(e);
        e.m.sb(ATTACKING, 0);
        self.enemy_c2_retarget_slot(e);
        self.enemy_c2_steer_slot(e);
        self.enemy_c2_ai_tail(e, h);
    }

    pub fn enemy_goto_fall(e: &mut Enemy) {
        e.b.set_state(0x0200_0C00);
    }

    /// 0x33F74 state 0: wait, maybe ambush.
    pub fn enemy_c2_decide(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.airborne != 0 {
            return Self::enemy_goto_fall(e);
        }
        let w = e.m.b(WAIT);
        if w != 0 {
            e.m.sb(WAIT, w - 1);
            if w - 1 == 0 {
                let thr = self.assets.enemy.t.u8(t2::AMBUSH, self.enemy_aggr()) as u16;
                if (self.rand() & 0xF) < thr {
                    self.enemy_c2_choose_combo(e, h);
                }
            }
            return;
        }
        e.b.act = 2;
        e.b.step = 0;
        self.enemy_c2_approach(e, h);
    }

    /// 0x35152 (buggy original: loads the player's first long, not its address; the value is
    /// overwritten at once by `c2_reset_ai`), then reset.
    pub fn enemy_c2_retarget_any(&mut self, e: &mut Enemy, h: Handle) {
        self.enemy_c2_reset_ai(e, h);
    }

    /// 0x33FB8 state 2: approach the slot.
    pub fn enemy_c2_approach(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.w(TARGET);
        let tb = self.body(t);
        // 0x33FB8: `cmpi.b #$2,$5(a6)` tests the enemy's own sub state (always 0 on this path),
        // not the target's (0x34398 tests $5(a0)). Quirk kept.
        if tb.live == 0 || e.b.sub == 2 {
            return self.enemy_c2_retarget_any(e, h);
        }
        if self.enemy_c2_bump_player(e) {
            e.b.act = 0xE;
            e.b.step = 0;
            self.enemy_anim(&mut e.b, 0);
            let v = self.assets.enemy.t.u8(t2::WAIT_E, self.enemy_aggr());
            e.m.sb(T80, v);
            return;
        }
        if self.enemy_c2_dash_trigger(e) {
            e.b.act = 0xA;
            e.b.step = 0;
            return;
        }
        if e.b.step == 0 {
            e.b.set_state(0x0200_0202);
            e.m.sb(ATTACKING, 0);
            e.m.sw(WB2, 0x12C);
            self.enemy_anim(&mut e.b, 1);
            e.m.sw(WA2, 0x36);
            e.m.sb(T80, 1);
            e.m.sw(WA0, 6);
            e.m.sb(DETOUR_T, 0);
            e.m.sb(DETOUR, 0);
        }
        if e.b.airborne != 0 {
            return Self::enemy_goto_fall(e);
        }
        if Self::enemy_near_slot(e) {
            return self.enemy_c2_at_slot(e, h);
        }
        let w = e.m.w(WA0);
        e.m.sw(WA0, w.wrapping_sub(1));
        if w == 0 {
            e.m.sw(WA0, 0x28);
            if e.m.b(DETOUR) == 0 {
                self.enemy_c2_steer_slot(e);
            }
        }
        if e.m.b(TAUNTED) == 0 {
            let w = e.m.w(WA2).wrapping_sub(1);
            e.m.sw(WA2, w);
            if w == 0 {
                e.m.sw(WA2, 0x24);
                let r = self.rand() & 0x1F;
                if (0x8A88_6551u32 >> r) & 1 != 0 {
                    e.m.sb(TAUNTED, 1);
                    e.b.act = 4;
                    e.b.step = 0;
                    return;
                }
            }
        }
        let d = e.m.b(DETOUR_T);
        e.m.sb(DETOUR_T, d.wrapping_sub(1));
        if d == 0 {
            e.m.sb(DETOUR_T, 0);
            e.m.sb(DETOUR, 0);
            if Self::enemy_near_slot(e) {
                if e.b.param_b == 0 {
                    self.enemy_step(&mut e.b);
                }
                return;
            }
        }
        if Self::enemy_stuck_check(e) {
            Self::enemy_rotate_slot(e);
        }
        self.enemy_move_dir(&mut e.b);
        self.enemy_walk_anim(&mut e.b);
    }

    /// 0x340AE state 4: taunt.
    pub fn enemy_c2_taunt(&mut self, e: &mut Enemy) {
        if e.b.step == 0 {
            self.enemy_anim(&mut e.b, 2);
            e.b.step += 2;
            let a = e.b.attacker;
            if a != 0 && self.body(a).id == 2 && self.rand() & 0x3F == 0 {
                // 0x2A5B6 ROM integrity check: always passes with a genuine ROM.
            }
        }
        if self.enemy_step(&mut e.b) {
            e.b.act = 2;
            e.b.step = 0;
            self.enemy_anim(&mut e.b, 1);
        }
    }

    /// 0x340FC state 6: hover at the slot (MAAI). +$07 never changes here, so the first-frame
    /// block runs every frame.
    pub fn enemy_c2_hover(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.step == 0 {
            e.m.sb(ATTACKING, 0);
            self.enemy_c2_walk_speed(e);
            e.m.sw(WA0, 0);
        }
        if e.b.airborne != 0 {
            return Self::enemy_goto_fall(e);
        }
        let t = e.m.b(T80).wrapping_sub(1);
        e.m.sb(T80, t);
        if t == 0 && Self::enemy_near_slot(e) {
            return self.enemy_c2_at_slot(e, h);
        }
        let p = e.m.b(PAUSE);
        e.m.sb(PAUSE, p.wrapping_sub(1));
        if p != 0 {
            return;
        }
        e.m.sb(PAUSE, 0);
        let w = e.m.w(WA0);
        e.m.sw(WA0, w.wrapping_sub(1));
        if w == 0 {
            let v = self.assets.enemy.t.u8(t2::PAUSE, self.rank.aggression as u32 & 7);
            e.m.sw(WA0, v as u16);
            self.enemy_c2_steer_slot(e);
        }
        if Self::enemy_near_slot(e) {
            let v = self.assets.enemy.t.u8(t2::PAUSE, self.w.level as u32 & 0x1F);
            e.m.sb(PAUSE, v);
            if e.b.param_b == 0 {
                self.enemy_step(&mut e.b);
            }
            return;
        }
        if Self::enemy_stuck_check(e) {
            Self::enemy_rotate_slot(e);
        }
        self.enemy_move_dir(&mut e.b);
        self.enemy_walk_anim(&mut e.b);
    }

    /// 0x353CE: at the slot, attack or reposition.
    pub fn enemy_c2_at_slot(&mut self, e: &mut Enemy, _h: Handle) {
        let v = Self::enemy_variant(e);
        let chance = self.assets.enemy.t.u8(t2::ATTACK_CHANCE, (v << 2) + (self.rank.aggression as u32 & 7));
        let r = (self.rand() & 0x1F) as u8;
        if chance > r {
            e.b.act = 8;
            e.b.step = 0;
            return;
        }
        e.m.sb(ATTACKING, 0);
        let mut c;
        let mut tries = 4i16;
        loop {
            let r = self.rand() & 0x1F;
            c = self.assets.enemy.t.u8(t2::REPOS_CHOICE, (v << 4) + r as u32);
            if c != e.m.b(REPOS) {
                break;
            }
            tries -= 1;
            if tries < 0 {
                break;
            }
        }
        e.m.sb(REPOS, c);
        let r = self.rand() & 0xF;
        e.m.sb(T80, self.assets.enemy.t.u8(t2::REPOS_TIME, r as u32));
        match c {
            0 => {
                let mut s = e.m.b(SLOT);
                if self.w.frame & 1 != 0 {
                    s = s.wrapping_add(1);
                    if s > 5 {
                        s = 0;
                    }
                } else {
                    s = s.wrapping_sub(1);
                    if (s as i8) < 0 {
                        s = 5;
                    }
                }
                e.m.sb(SLOT, s);
                self.enemy_c2_steer_slot(e);
            }
            1 => {
                e.b.act = 6;
                e.b.step = 0;
            }
            _ => {
                let t = self.body(e.m.w(TARGET));
                let d = Self::enemy_angle_to(&e.b, t.xi() as u16, t.zi() as u16);
                e.b.dir = d.wrapping_add(0x10) & 0x1F;
                e.m.sb(DETOUR_T, 0x32);
                e.m.sb(DETOUR, 1);
            }
        }
    }

    /// 0x35172: new target and starting slot (QUIRK kept: see SPEC 7.4).
    pub fn enemy_c2_retarget_slot(&mut self, e: &mut Enemy) {
        let t = self.enemy_pick_target(&e.b);
        e.m.sw(TARGET, t);
        let p = ((t - Game::player_handle(0)) / 0x180) as usize;
        let c = self.spawn.g.census_prev[p];
        let l = c[2].wrapping_sub(1) as i8;
        let r = c[3].wrapping_sub(1);
        let tb = self.body(t);
        let mut s: u8 = if l > r as i8 {
            3
        } else if l < r as i8 {
            r
        } else if neg_near(e.b.xi().wrapping_sub(tb.xi())) {
            r
        } else {
            3
        };
        let mut d2: i16 = 0x13;
        if (e.b.zi() as u16) < (tb.zi() as u16) {
            s = s.wrapping_add(1);
            d2 = -0x13;
        }
        if (tb.zi().wrapping_add(d2) as u16) >= (e.b.zi() as u16) {
            s = s.wrapping_add(1);
        }
        e.m.sb(SLOT, s);
    }

    /// 0x351C4: slot point and turn toward it (max 4 steps).
    pub fn enemy_c2_steer_slot(&mut self, e: &mut Enemy) {
        let t = self.body(e.m.w(TARGET)).clone();
        let mut s = e.m.b(SLOT) as u32;
        if t.facing != 0 {
            s = (s as u8).wrapping_add(3) as u32;
            if s >= 6 {
                s -= 6;
            }
        }
        let tt = &self.assets.enemy.t;
        let mut dx = tt.i16(t2::SLOTS, (s & 0xFF) * 4);
        let dz = tt.i16(t2::SLOTS, (s & 0xFF) * 4 + 2);
        if t.facing != 0 {
            dx = dx.wrapping_neg();
        }
        let x = t.xi().wrapping_add(dx) as u16;
        let z = t.zi().wrapping_add(dz) as u16;
        e.m.sw(DEST_X, x);
        e.m.sw(DEST_Z, z);
        let d = Self::enemy_angle_to(&e.b, x, z);
        let diff = e.b.dir.wrapping_sub(d).wrapping_add(4) & 0x1F;
        if diff <= 8 {
            e.b.dir = d & 0x1F;
        } else {
            let step: u8 = if diff >= 0x14 { 4 } else { 0xFC };
            e.b.dir = e.b.dir.wrapping_add(step) & 0x1F;
        }
        e.b.facing = !neg_near(t.xi().wrapping_sub(e.b.xi())) as u8;
    }

    /// 0x35264: slot point (no mirroring) and direction with no turn limit (dash start).
    pub fn enemy_c2_aim_slot(&mut self, e: &mut Enemy) {
        let t = self.body(e.m.w(TARGET)).clone();
        let s = e.m.b(SLOT) as u32;
        let tt = &self.assets.enemy.t;
        let x = tt.i16(t2::SLOTS, s * 4).wrapping_add(t.xi()) as u16;
        let z = tt.i16(t2::SLOTS, s * 4 + 2).wrapping_add(t.zi()) as u16;
        e.m.sw(DEST_X, x);
        e.m.sw(DEST_Z, z);
        e.b.dir = Self::enemy_angle_to(&e.b, x, z);
    }

    /// 0x35296: attack position (55 px from the target) and direction; face the target.
    pub fn enemy_c2_steer_attack(&mut self, e: &mut Enemy) {
        let t = self.body(e.m.w(TARGET)).clone();
        let mut s = e.m.b(SLOT) as u32;
        if t.facing != 0 {
            s = (s as u8).wrapping_add(3) as u32;
            if s >= 6 {
                s -= 6;
            }
        }
        let tt = &self.assets.enemy.t;
        let mut dx = tt.i16(t2::ATTACK_POS, s * 4);
        let dz = tt.i16(t2::ATTACK_POS, s * 4 + 2);
        if t.facing != 0 {
            dx = dx.wrapping_neg();
        }
        let x = t.xi().wrapping_add(dx) as u16;
        let z = t.zi().wrapping_add(dz) as u16;
        e.m.sw(DEST_X, x);
        e.m.sw(DEST_Z, z);
        e.b.dir = Self::enemy_angle_to(&e.b, x, z);
        e.b.facing = !neg_near(t.xi().wrapping_sub(e.b.xi())) as u8;
    }

    /// 0x3535C.
    pub fn enemy_near_slot(e: &Enemy) -> bool {
        in_win(e.b.xi().wrapping_sub(e.m.w(DEST_X) as i16), 0x18) && (e.b.zi().wrapping_sub(e.m.w(DEST_Z) as i16).wrapping_add(0xF) as u16) < 0x1E
    }

    /// 0x3538A.
    pub fn enemy_at_attack_pos(e: &Enemy) -> bool {
        in_win(e.b.xi().wrapping_sub(e.m.w(DEST_X) as i16), 4) && (e.b.zi().wrapping_sub(e.m.w(DEST_Z) as i16).wrapping_add(5) as u16) < 0xA
    }

    /// 0x357F6: rotate the slot by 2 and flip the direction (stuck).
    pub fn enemy_rotate_slot(e: &mut Enemy) {
        let mut s = e.m.b(SLOT).wrapping_add(2);
        if (s as i8) >= 6 {
            s = s.wrapping_sub(6);
        }
        e.m.sb(SLOT, s);
        e.b.dir ^= 0x10;
    }

    /// 0x3555A: a free player right in front: he becomes the target.
    pub fn enemy_c2_bump_player(&self, e: &mut Enemy) -> bool {
        for p in [2usize, 1, 0] {
            let q = &self.players[p].b;
            if q.live == 0 || q.sub != 0 {
                continue;
            }
            if !in_win(q.xi().wrapping_sub(e.b.xi()), 0x18) {
                continue;
            }
            if (q.zi().wrapping_sub(e.b.zi()).wrapping_add(2) as u16) < 5 {
                e.m.sw(TARGET, Game::player_handle(p));
                return true;
            }
        }
        false
    }

    /// State 0xE (0x34204): short pause, then attack.
    pub fn enemy_c2_wait_attack(&mut self, e: &mut Enemy) {
        let t = e.m.b(T80);
        e.m.sb(T80, t.wrapping_sub(1));
        if t == 0 {
            e.b.set_state(0x0200_0800);
        }
    }

    /// 0x355C0: start a dash (state A) toward a far player.
    pub fn enemy_c2_dash_trigger(&mut self, e: &mut Enemy) -> bool {
        let t = self.body(e.m.w(TARGET)).clone();
        if t.sub != 0 {
            return false;
        }
        let dx = t.xi().wrapping_sub(e.b.xi());
        let mask: u32;
        if !in_win(dx, 0xC8) {
            let c = e.m.b(DASH_T).wrapping_sub(1);
            e.m.sb(DASH_T, c);
            if c != 0 {
                return false;
            }
            e.m.sb(DASH_T, 0x14);
            e.m.sb(DASH_KIND, 1);
            mask = 0x4541;
        } else {
            if t.act != 2 || in_win(dx, 0xA0) {
                return false;
            }
            let c = e.m.b(DASH_T).wrapping_sub(1);
            e.m.sb(DASH_T, c);
            if c != 0 {
                return false;
            }
            e.m.sb(DASH_T, 0xA);
            e.m.sb(DASH_KIND, 0);
            mask = 0x914B;
        }
        let r = self.rand() & 0xF;
        if (mask >> r) & 1 == 0 {
            return false;
        }
        e.b.facing = !neg_near(t.xi().wrapping_sub(e.b.xi())) as u8;
        true
    }

    /// 0x34610 state A: dash.
    pub fn enemy_c2_dash(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.airborne != 0 {
            return Self::enemy_goto_fall(e);
        }
        match e.b.step {
            0 | 2 => {
                if e.b.step == 0 {
                    e.b.step = 2;
                    e.m.sb(ATTACKING, 0);
                    self.enemy_anim(&mut e.b, 6);
                    self.enemy_set_speed(&mut e.b, 0x1F);
                    e.m.sb(TURNED, 0);
                    let t = self.body(e.m.w(TARGET));
                    e.b.facing = !neg_near(t.xi().wrapping_sub(e.b.xi())) as u8;
                    e.m.sw(WA0, 0);
                    self.enemy_c2_aim_slot(e);
                }
                if self.enemy_c2_dash_attack_check(e) {
                    return self.enemy_c2_start_dash_attack(e);
                }
                if self.enemy_c2_dash_stop(e) {
                    return self.enemy_c2_reset_ai(e, h);
                }
                let w = e.m.w(WA0);
                e.m.sw(WA0, w.wrapping_sub(1));
                if w == 0 {
                    e.m.sw(WA0, 0xA);
                    self.enemy_c2_turn_to_target(e);
                    let want = (e.b.dir < 0x10) as u8;
                    if want != e.b.facing {
                        if e.m.b(TURNED) != 0 {
                            return self.enemy_c2_reset_ai(e, h);
                        }
                        e.b.facing = want;
                        e.m.sb(TURNED, 1);
                    }
                }
                self.enemy_move_dir(&mut e.b);
                self.enemy_step(&mut e.b);
            }
            _ => {
                if e.b.airborne != 0 {
                    return Self::enemy_goto_fall(e);
                }
                if e.b.hitstop != 0 {
                    e.b.hitstop -= 1;
                    if e.b.hitstop != 0 {
                        return;
                    }
                    e.b.timer = 1;
                }
                let t = e.m.b(T80).wrapping_sub(1);
                e.m.sb(T80, t);
                if t == 0 {
                    return self.enemy_c2_reset_ai(e, h);
                }
                if !in_win(e.b.vx, 0x30) {
                    self.enemy_step(&mut e.b);
                    e.b.motion();
                } else {
                    self.enemy_c2_reset_ai(e, h);
                }
            }
        }
    }

    /// 0x3565C: point at the target, turn max 2 steps.
    pub fn enemy_c2_turn_to_target(&mut self, e: &mut Enemy) {
        let t = self.body(e.m.w(TARGET)).clone();
        e.m.sw(DEST_X, t.xi() as u16);
        e.m.sw(DEST_Z, t.zi() as u16);
        let d = Self::enemy_angle_to(&e.b, t.xi() as u16, t.zi() as u16);
        let diff = e.b.dir.wrapping_sub(d).wrapping_add(2) & 0x1F;
        if diff <= 4 {
            e.b.dir = d & 0x1F;
        } else {
            let s: u8 = if diff >= 0x12 { 2 } else { 0xFE };
            e.b.dir = e.b.dir.wrapping_add(s) & 0x1F;
        }
    }

    /// 0x346C0: dash attack (atk 0x0C) with friction.
    pub fn enemy_c2_start_dash_attack(&mut self, e: &mut Enemy) {
        e.b.ax = e.b.vx.wrapping_neg() >> 6;
        e.b.az = e.b.vz.wrapping_neg() >> 6;
        e.b.vy = 0;
        e.b.ay = 0;
        e.m.sb(T80, 0x3C);
        e.b.step = 4;
        self.enemy_anim(&mut e.b, 7);
        self.enemy_begin_attack(&mut e.b, 0x0C);
        // 0x3470E runs in the same frame (falls through).
        if e.b.airborne != 0 {
            return Self::enemy_goto_fall(e);
        }
        let t = e.m.b(T80).wrapping_sub(1);
        e.m.sb(T80, t);
        if !in_win(e.b.vx, 0x30) {
            self.enemy_step(&mut e.b);
            e.b.motion();
        }
    }

    /// 0x2B56C (table by +$84): the class' own "back to the AI" routine. Ported: 0xC, 0x28, 0x2C
    /// (class 10, 0x3BC38), 0x20 / 0x24 (class 20, 0x45D74), 0x18 / 0x50 (class 14, 0x408E8). Entries of classes that are not ported yet: back to AI state 0.
    pub fn enemy_recover_other(&mut self, e: &mut Enemy, h: Handle, w84: u16) {
        match w84 {
            0x0C | 0x28 | 0x2C => self.c10_reset_ai(e, h),
            0x30 | 0x34 => self.enemy_vice_recover(e), // 0x2B59C -> 0x42AE4
            0x20 | 0x24 => self.c20_reset_ai(e, h), // 0x45D74
            0x18 | 0x50 => self.c14_408e8(e),       // 0x408E8 class 14
            0x08 | 0x48 | 0x4C => self.enemy_c13_3e504(e), // 0x3E504 the poachers (class 13)
            0x38 => self.c3_reset_ai(e, h),                 // 0x35D36 LASH T. (class 3)
            0x78 | 0x7C | 0x80 => self.c79_enter_ai(e, h), // 0x5F84C the dinosaur hunters (class 79)
            0x3C => self.c61_decide(e, h),          // 0x58D40 the Butcher (class 61)
            0x14 => self.c25_alive_start(e, h),     // 0x4C072 Slice (class 25)
            0x70 => self.c63_recover(e),            // 0x5ACAA the alternate Blade (class 63)
            0x74 | 0x90 => self.c64_recover(e),     // 0x5B2C4 the bomb carriers (class 64)
            0x54 => self.enemy_morgan_recover(e), // 0x51ACA
            0x40 => self.enemy_c6_reset_ai(e),      // 0x396C2 Bludge (class 6)
            _ => e.b.set_state(0x0200_0000),
        }
    }

    /// +$58 = id, +$5A = 0, +$59 = 0, +$5E = next attack serial.
    pub fn enemy_begin_attack(&mut self, b: &mut Body, id: u8) {
        b.atk = id;
        b.knock = 0;
        b.hitstop = 0;
        b.serial = self.w.attack_serial;
        self.w.attack_serial = self.w.attack_serial.wrapping_add(1);
    }

    /// 0x35702: turn the dash into a dash attack.
    pub fn enemy_c2_dash_attack_check(&mut self, e: &mut Enemy) -> bool {
        if e.m.b(NO_DASH_ATK) != 0 {
            return false;
        }
        let t = self.body(e.m.w(TARGET)).clone();
        if t.live == 0 || t.sub != 0 {
            return false;
        }
        let px = t.xi().wrapping_add(if t.xi() as u16 >= e.b.xi() as u16 { -0x60 } else { 0x60 });
        if !in_win(px.wrapping_sub(e.b.xi()), 0x40) {
            return false;
        }
        if (t.zi().wrapping_sub(e.b.zi()).wrapping_add(8) as u16) >= 0x10 {
            return false;
        }
        let p = self.assets.enemy.t.u8(t2::DASH_ATTACK, (Self::enemy_variant(e) << 4) + self.w.level as u32);
        let r = (self.rand() & 0x1F) as u8;
        if r >= p {
            return false;
        }
        if (e.b.dir.wrapping_sub(6) & 0xF) < 5 {
            // 0x35782: `move.w a0,$76(a6)` with a0 = the table pointer of 0x35758 (the target is not
            // reloaded): +$76 holds the low word of 0x35796 + variant * 16 (0x5796 / 0x57B6 / 0x57D6)
            // until the next target pick. Quirk kept.
            e.m.sw(TARGET, (0x35796u32 + (Self::enemy_variant(e) << 4)) as u16);
            return true;
        }
        e.m.sb(NO_DASH_ATK, 1);
        false
    }

    /// 0x356A2: stop the dash (every 31 frames).
    pub fn enemy_c2_dash_stop(&mut self, e: &mut Enemy) -> bool {
        let c = e.m.b(DASH_CHK);
        e.m.sb(DASH_CHK, c.wrapping_sub(1));
        if c != 0 {
            return false;
        }
        e.m.sb(DASH_CHK, 0x1E);
        let t = self.body(e.m.w(TARGET)).clone();
        let stop = if in_win(t.xi().wrapping_sub(e.b.xi()), 0x70) {
            true
        } else if in_win(e.b.xi().wrapping_sub(e.m.w(DASH_X) as i16), 0x10) {
            true
        } else if e.m.b(DASH_KIND) != 0 || t.sub != 0 {
            false
        } else {
            t.act != 2
        };
        e.m.sw(DASH_X, e.b.xi() as u16);
        stop
    }

    /// 0x34222 state 8: attack.
    pub fn enemy_c2_attack(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.hitstop != 0 {
            e.b.hitstop -= 1;
            if e.b.hitstop != 0 {
                return;
            }
            e.b.timer = 1;
        }
        if e.b.step != 0xE && e.b.airborne != 0 {
            return Self::enemy_goto_fall(e);
        }
        match e.b.step {
            0 => {
                e.b.set_state(0x0200_0802);
                e.m.sb(ATTACKING, 1);
                self.enemy_anim(&mut e.b, 1);
                e.m.sw(WA0, 0);
                // 0x35310 attack side
                let t = self.body(e.m.w(TARGET)).clone();
                let base: u16 = if neg_near(e.b.xi().wrapping_sub(t.xi())) { 0 } else { 3 };
                let r = self.rand() & 0xF;
                let s = base.wrapping_add(self.assets.enemy.t.u16(t2::SIDE, 2 * r as u32));
                e.m.sb(SLOT, s as u8);
                e.m.sb(T80, 0xF0);
                self.enemy_c2_run_speed(e);
                self.enemy_c2_attack_approach(e, h);
            }
            2 => self.enemy_c2_attack_approach(e, h),
            4 => self.enemy_c2_choose_combo(e, h),
            6 => self.enemy_c2_combo_step(e, h),
            8 | 0xA | 0xC => {
                if self.enemy_step(&mut e.b) {
                    self.enemy_c2_recovery(e, h);
                }
            }
            0xE => self.enemy_c2_jump_run(e, h),
            _ => {
                let t = e.m.b(T80);
                e.m.sb(T80, t.wrapping_sub(1));
                if t == 0 {
                    self.enemy_c2_combo_step(e, h);
                }
            }
        }
    }

    /// 0x34290.
    pub fn enemy_c2_attack_approach(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.b(T80);
        e.m.sb(T80, t.wrapping_sub(1));
        if t == 0 {
            return self.enemy_c2_reset_ai(e, h);
        }
        let w = e.m.w(WA0);
        e.m.sw(WA0, w.wrapping_sub(1));
        if w == 0 {
            e.m.sw(WA0, 6);
            self.enemy_c2_steer_attack(e);
        }
        if Self::enemy_at_attack_pos(e) {
            if e.b.param_b == 0 {
                self.enemy_walk_anim(&mut e.b);
            }
            e.b.step = 4;
            return self.enemy_c2_choose_combo(e, h);
        }
        if Self::enemy_stuck_check(e) {
            Self::enemy_rotate_slot(e);
        }
        self.enemy_move_dir(&mut e.b);
        self.enemy_walk_anim(&mut e.b);
    }

    /// 0x342D8: pick a combo (or give up when the target shows its back).
    pub fn enemy_c2_choose_combo(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.wall & 0x80 != 0 {
            return self.enemy_c2_retarget_any(e, h);
        }
        let t = self.body(e.m.w(TARGET)).clone();
        if e.b.facing == t.facing {
            let r = self.rand() & 0x1F;
            if r >= 7 {
                return self.enemy_c2_reset_ai(e, h);
            }
        }
        let r = self.rand() & 0x1F;
        let c = self.assets.enemy.t.u8(t2::COMBO_CHOICE, (Self::enemy_variant(e) << 4) + r as u32);
        e.m.sb(COMBO, c);
        e.m.sw(CURSOR, c as u16 * 6);
        e.b.step = 6;
        self.enemy_c2_combo_step(e, h);
    }

    /// 0x34398: next action of the combo.
    pub fn enemy_c2_combo_step(&mut self, e: &mut Enemy, h: Handle) {
        let t = self.body(e.m.w(TARGET)).clone();
        if t.live == 0 || t.sub == 2 || t.sub == 6 {
            return self.enemy_c2_reset_ai(e, h);
        }
        if e.b.wall & 0x80 == 0 && !self.enemy_c2_in_attack_range(e, &t) {
            e.b.act = 0;
            e.b.step = 0;
            return;
        }
        let a = self.assets.enemy.t.u8(t2::COMBOS, e.m.w(CURSOR) as u32);
        if (a as i8) < 0 {
            return self.enemy_c2_choose_combo(e, h);
        }
        e.m.sw(CURSOR, e.m.w(CURSOR).wrapping_add(1));
        match a {
            0 => self.enemy_c2_act(e, h, 0x0808, 3, 0),
            2 => self.enemy_c2_act(e, h, 0x080A, 4, 1),
            4 => self.enemy_c2_act(e, h, 0x080C, 5, 2),
            _ => self.enemy_c2_jump_attack(e, h),
        }
    }

    /// 0x35512: the target is 0x34 px in front of it on our side.
    pub fn enemy_c2_in_attack_range(&self, e: &Enemy, t: &Body) -> bool {
        let d3: i16 = if neg_near(e.b.xi().wrapping_sub(t.xi())) { -0x34 } else { 0x34 };
        let px = t.xi().wrapping_add(d3);
        in_win(e.b.xi().wrapping_sub(px), 0x14) && (e.b.zi().wrapping_sub(t.zi()).wrapping_add(5) as u16) < 0xA
    }

    /// 0x34420 / 0x34466 / 0x344AC: punch actions.
    pub fn enemy_c2_act(&mut self, e: &mut Enemy, h: Handle, st: u16, anim: u16, atk: u8) {
        e.b.act = (st >> 8) as u8;
        e.b.step = st as u8;
        self.enemy_anim(&mut e.b, anim);
        self.enemy_begin_attack(&mut e.b, atk);
        if (self.assets.enemy.t.u8(t2::COMBOS, e.m.w(CURSOR) as u32) as i8) < 0 {
            e.b.knock = 1;
        }
        if self.enemy_step(&mut e.b) {
            self.enemy_c2_recovery(e, h);
        }
    }

    /// 0x344F2: jump attack (atk 3).
    pub fn enemy_c2_jump_attack(&mut self, e: &mut Enemy, h: Handle) {
        e.b.act = 8;
        e.b.step = 0xE;
        e.m.sb(B90, 0);
        self.enemy_anim(&mut e.b, 8);
        self.enemy_begin_attack(&mut e.b, 3);
        let t = self.body(e.m.w(TARGET)).clone();
        let left = neg_near(t.xi().wrapping_sub(e.b.xi()).wrapping_add(0)) ;
        // 0x3451C: d1 = target X - X + 0x400; carry -> vx = -0xA0, facing 0.
        let (vx, f) = if left { (-0xA0, 0) } else { (0xA0, 1) };
        e.b.vx = vx;
        e.b.facing = f;
        e.b.vy = 0x480;
        e.b.ay = -0x30;
        e.b.ax = 0;
        e.b.vz = 0;
        e.b.az = 0;
        e.b.airborne = 1;
        self.enemy_c2_jump_run(e, h);
    }

    /// 0x34560.
    pub fn enemy_c2_jump_run(&mut self, e: &mut Enemy, h: Handle) {
        if e.m.b(B90) == 0 {
            self.enemy_step(&mut e.b);
            if e.b.param_a == 0 {
                e.b.motion();
                if e.b.airborne == 0 {
                    e.m.sb(B90, 2);
                    self.enemy_anim(&mut e.b, 9);
                }
            }
        } else if self.enemy_step(&mut e.b) {
            self.enemy_c2_recovery(e, h);
        }
    }

    /// 0x3459A: pause between hits (by level band).
    pub fn enemy_c2_recovery(&mut self, e: &mut Enemy, h: Handle) {
        e.b.act = 8;
        e.b.step = 0x10;
        let r = self.rand() & 0x1F;
        let i = self.assets.enemy.t.u8(t2::RECOVER_IDX, r as u32);
        let band = self.w.level as u32 & 0x18;
        let v = self.assets.enemy.t.u8(t2::RECOVER, band + i as u32);
        e.m.sb(T80, v);
        self.enemy_anim(&mut e.b, 1);
        let t = e.m.b(T80);
        e.m.sb(T80, t.wrapping_sub(1));
        if t == 0 {
            self.enemy_c2_combo_step(e, h);
        }
    }

    /// 0x34758 state C: fall.
    pub fn enemy_c2_fall(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.step {
            0 => {
                e.b.step = 2;
                self.enemy_anim(&mut e.b, 0xC);
                e.b.vx = 0;
                e.b.ax = 0;
                e.b.vy = 0;
                e.b.ay = -0x20;
                e.b.vz = 0;
                e.b.az = 0;
            }
            2 => {
                if e.b.airborne != 0 {
                    e.b.motion();
                    self.enemy_step(&mut e.b);
                    return;
                }
                e.b.step = 4;
                self.enemy_anim(&mut e.b, 9);
                if self.enemy_step(&mut e.b) {
                    self.enemy_c2_reset_ai(e, h);
                }
            }
            _ => {
                if self.enemy_step(&mut e.b) {
                    self.enemy_c2_reset_ai(e, h);
                }
            }
        }
    }

    /// 0x347C8 state 0x10: hop down a ledge.
    pub fn enemy_c2_hop(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.step {
            0 => {
                e.b.step = 2;
                self.enemy_anim(&mut e.b, 0x10);
                e.b.vx = 0;
                e.b.ax = 0;
                e.b.vy = 0x700;
                e.b.ay = -0x50;
                e.b.vz = 0x20;
                e.b.az = 0;
            }
            2 => {
                if e.b.param_b != 0 {
                    self.enemy_step(&mut e.b);
                    return;
                }
                e.b.airborne = 1;
                e.b.step = 4;
                self.enemy_c2_hop_land(e, h);
            }
            _ => self.enemy_c2_hop_land(e, h),
        }
    }

    pub fn enemy_c2_hop_land(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.step == 4 {
            if e.b.airborne != 0 {
                e.b.motion();
                self.enemy_step(&mut e.b);
                return;
            }
            e.b.step = 6;
            self.enemy_anim(&mut e.b, 9);
        }
        if self.enemy_step(&mut e.b) {
            self.enemy_c2_reset_ai(e, h);
        }
    }
}

/// 0x10036 / 0x10008: BCD long add with `abcd`, saturating at 99999999.
pub fn bcd_add_long(a: u32, b: u32) -> u32 {
    let mut r = 0u32;
    let mut carry = 0u32;
    for k in 0..8 {
        let d = ((a >> (4 * k)) & 0xF) + ((b >> (4 * k)) & 0xF) + carry;
        carry = (d > 9) as u32;
        r |= (if d > 9 { d - 10 } else { d }) << (4 * k);
    }
    if carry != 0 { 0x9999_9999 } else { r }
}

impl Body {
    /// Sets the integer part of X, keeping the fraction (`sub.w d0,$8(a6)`).
    pub fn set_xi_keep(&mut self, v: i16) {
        self.x = (self.x & 0xFFFF) | ((v as i32) << 16);
    }
    pub fn set_zi_keep(&mut self, v: i16) {
        self.z = (self.z & 0xFFFF) | ((v as i32) << 16);
    }
    pub fn set_yi_keep(&mut self, v: i16) {
        self.y = (self.y & 0xFFFF) | ((v as i32) << 16);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(x: i16, z: i16) -> Body {
        let mut b = Body::default();
        b.set_xi(x);
        b.set_zi(z);
        b
    }

    #[test]
    fn angles() {
        let b = at(100, 100);
        assert_eq!(Game::enemy_angle_to(&b, 100, 200), 0); // +Z
        assert_eq!(Game::enemy_angle_to(&b, 200, 100), 8); // +X
        assert_eq!(Game::enemy_angle_to(&b, 100, 0), 0x10); // -Z
        assert_eq!(Game::enemy_angle_to(&b, 0, 100), 0x18); // -X
        assert_eq!(Game::enemy_angle_to(&b, 200, 200), 4);
    }

    /// 0x10B2 adds with `add.b`: for dx 0x3F8 and dz 0x13E the accumulator (0x7F, step 0xFE) stays below dz
    /// when only its low byte changes, so the sub-step is 4 (a word add would stop at 1 and give 0x17). Found by
    /// the c10a recording of MAME, frame 467 (final direction 0x15 after one step of the turn).
    #[test]
    fn angle_adds_with_a_byte() {
        let b = at(0x4B3, 0x13E);
        assert_eq!(Game::enemy_angle_to(&b, 0xBB, 0), 0x14);
    }

    #[test]
    fn windows() {
        assert!(in_win(-0x17, 0x18));
        assert!(!in_win(0x18, 0x18));
        assert!(neg_near(-1));
        assert!(!neg_near(0));
    }
}
