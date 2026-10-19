//! Placeholders for pool A classes the port does not run yet (M3-01). They keep the wave, lock,
//! GO arrow, clock and drop flow alive until the real class is ported. They are not exact:
//! a walker walks to the nearest hero and swings at him, a hazard crosses the screen, and every
//! placeholder is drawn as a labelled box (`draw_boxes`, called by `present.rs`).
//! The stage bosses (`Spec::boss`) are the same objects with the boss inits (HP row, controller, the death chain 0xACEE,
//! Morgan's rebirth as Morgue). The boss starters (pool B 33 and 79) are placeholders too (`ph_starter`).
//!
//! The registry `SPECS` lists the classes served by placeholders. A real port deletes its entry
//! (`--list-placeholders` prints the list). The census flag, the name ids and the kinds come from
//! `plan/game/design/D3a-humans.md` s1 and s13, `D3b-dinosaurs.md` s1 and s9, and the name ids of
//! `re/specs/enemy/classmap.txt` (`move.b #id,$3(a6)`). Owner: enemy port.

use super::body::Body;
use super::enemy::{Enemy, off::*};
use super::prop::Prop;
use super::{Game, Handle};
use crate::render::DrawTarget;
use dino_assets::enemy::SpawnRec;
use macroquad::prelude::{Color, BLACK};

/// What the placeholder knows of one class.
pub struct Spec {
    pub class: u16,
    /// Short name for the list and the box.
    pub what: &'static str,
    /// Name id (+$3, row of the kill score table 0x325F8). None: no kill score.
    pub name: Option<u8>,
    /// Name id when the variant (+$26 high byte) is 2 (class 18: Walther). None: `name`.
    pub name_v2: Option<u8>,
    /// Census flag: counted by the waves (+$7D = 0), or not (0xFF).
    pub counted: bool,
    /// Dinosaur: freed when the camera reaches its leave X (+$5C, 0 = never).
    pub dino: bool,
    /// Hazard or harmless creature: crosses the screen and never attacks or walks to a hero.
    pub hazard: bool,
    /// Scene actor (openings, the Cadillac): stands where it was made and never attacks.
    pub still: bool,
    /// Boss part (variant 0 only, `boss_of`): a stage boss, not counted, with the death chain (0xACEE).
    pub boss: Option<Boss>,
}

/// The boss part of a placeholder: what the boss inits of the original set (HP row, controller, score, the rebirth).
#[derive(Clone, Copy)]
pub struct Boss {
    /// HP row of the table at 0x31E14 (`enemy_set_boss_hp`): Vice and Slice 0, Butcher 1, Hogg 2, Morgan 4, Morgue 5,
    /// Tyrog 6, Fessenden (class 94) 8, the Slisaurs 0.
    pub hp_row: usize,
    /// Its init makes the boss controller (the reinforcements, `boss.rs`).
    pub controller: bool,
    /// The death gives the kill score of the name (0x325E6). Morgan gives none.
    pub score: bool,
    /// At the death the same object becomes this class and the fight goes on (Morgan becomes Morgue, 0x52EBE).
    pub rebirth: Option<u16>,
    /// One of a group (the three Slisaurs): its death ends the fight only when no other of its class is alive.
    pub group: bool,
}

const fn boss(hp_row: usize, controller: bool) -> Option<Boss> {
    Some(Boss { hp_row, controller, score: true, rebirth: None, group: false })
}

const fn walker(class: u16, what: &'static str, name: Option<u8>, counted: bool) -> Spec {
    Spec { class, what, name, name_v2: None, counted, dino: false, hazard: false, still: false, boss: None }
}

const fn dino(class: u16, what: &'static str, name: Option<u8>) -> Spec {
    Spec { class, what, name, name_v2: None, counted: false, dino: true, hazard: false, still: false, boss: None }
}

const fn cross(class: u16, what: &'static str, counted: bool) -> Spec {
    Spec { class, what, name: None, name_v2: None, counted, dino: false, hazard: true, still: false, boss: None }
}

/// Every pool A class of `stage_scripts.txt` that the port does not run yet.
pub const SPECS: &[Spec] = &[
    walker(3, "LASH T. (mace)", Some(0x0E), true),
    // Class 5 (M6-09) and class 61 (M6-10), the Butcher, are real.
    walker(6, "BLUDGE", Some(0x10), true),
    // Class 13 (M6-01 / M6-02), the poachers, is real.
    dino(22, "BRONTOSAURUS foot", None),
    // Class 24 (M6-04, M6-05), Shivat, is real. Class 25 (M7-01 .. M7-04), Slice, is real.
    Spec { boss: Some(Boss { hp_row: 4, controller: true, score: false, rebirth: Some(40), group: false }), ..walker(35, "MORGAN", Some(0x15), true) },
    // Class 38 (M6-03), Mack, is real.
    walker(45, "MORGAN villager", None, true),
    Spec { boss: boss(5, true), ..walker(40, "MORGUE", Some(0x16), false) },
    Spec { boss: boss(6, false), ..walker(4, "TYROG", Some(0x19), false) },
    // Classes 78 and 80 (M6-03), the ground critters, are real.
    walker(79, "HUNTER", Some(0x1E), true),
    Spec { boss: Some(Boss { hp_row: 0, controller: false, score: true, rebirth: None, group: true }), ..walker(71, "SLISAUR", Some(0x18), false) },
    Spec { boss: boss(8, false), ..walker(94, "FESSENDEN form 2", Some(0x25), true) },
];

/// The spec of a placeholder class, None when the port runs the class (or it has no placeholder).
pub fn spec_of(class: u16) -> Option<&'static Spec> {
    SPECS.iter().find(|s| s.class == class)
}

/// Text of `--list-placeholders`: one line per class.
pub fn list_text() -> String {
    let mut s = String::from("class  kind      census  name   what\n");
    for sp in SPECS {
        let kind = if sp.hazard { "hazard" } else if sp.dino { "dinosaur" } else if sp.still { "actor" } else { "walker" };
        let census = if sp.counted { "counted" } else { "not" };
        let name = sp.name.map_or("-".to_string(), |n| format!("{n:#04x}"));
        s.push_str(&format!("{:>5}  {kind:<8}  {census:<7} {name:<6} {}\n", sp.class, sp.what));
    }
    s
}

/// Variant 0 of a boss class is the boss (Vice T.): not counted by the census (`+$7D` = 0xFF), see `boss.rs`.
pub fn boss_of(spec: &Spec, var26: u16) -> bool {
    boss_part(spec, var26).is_some()
}

/// The boss part of a placeholder object, None when it is not a boss.
pub fn boss_part(spec: &Spec, var26: u16) -> Option<Boss> {
    if (var26 >> 8) == 0 { spec.boss } else { None }
}

/// Animation table of class 2 (variant 0): the placeholder takes its body and attack boxes from it.
const ANIM_TABLE: u32 = 0x34892;
/// Box table of class 2 (the body box table of the common enemy code).
const BOX_TABLE: u32 = 0x103000;
/// Hit points of every placeholder (the common enemy HP row is not known for these classes).
const HP: u16 = 44;
/// Walk speed 1.5 px per frame in 16.16 positions.
const STEP: i32 = 0x18000;
/// Reach of the swing: 48 px in X, 24 px in Z.
const REACH_X: u32 = 0x30 << 16;
const REACH_Z: u32 = 0x18 << 16;
/// Frames between two swings.
const COOLDOWN: u8 = 90;
/// Frames of the death blink.
const BLINK: u16 = 30;
/// Own bytes of the object (raw bytes beyond the header; no other class of the placeholders uses them).
const SWING: usize = 0xD0; // 1 while the swing animation runs
const DIR: usize = 0xD1; // hazards: 1 = crosses to the right

impl Game {
    /// Placeholder update of a registered class (called by `enemy_class_unported`).
    pub fn enemy_ph_update(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        let Some(spec) = spec_of(e.b.k20 / 4) else {
            return self.enemy_free_pool_a(e, i);
        };
        match e.b.mode {
            0 => self.enemy_ph_init(e, h, spec),
            2 => {
                self.enemy_ph_alive(e, h, i, spec);
            }
            4 => self.enemy_ph_dying(e, h, spec),
            _ => self.enemy_free_pool_a(e, i),
        }
    }

    fn enemy_ph_init(&mut self, e: &mut Enemy, h: Handle, spec: &Spec) {
        self.enemy_ph_setup(e, h, spec, true);
        e.b.mode += 2;
    }

    /// The init of a placeholder. `first` is false for a rebirth (Morgue in Morgan's object): the controller and the
    /// HUD bar stay those of the first form.
    fn enemy_ph_setup(&mut self, e: &mut Enemy, h: Handle, spec: &Spec, first: bool) {
        let name = if (e.b.var26 >> 8) == 2 { spec.name_v2.or(spec.name) } else { spec.name };
        e.b.id = name.unwrap_or(0);
        // +$84 (per-type rows of the hit, grab and damage tables) stays at FERRIS's row 0: the grab
        // and damage rows of the other names are not extracted yet. The name (+$3) gives the score.
        e.m.sw(W84, 0);
        e.b.anim_table = ANIM_TABLE;
        e.b.box_table = BOX_TABLE;
        e.b.multi_box = false;
        self.enemy_anim(&mut e.b, 0);
        let boss = boss_part(spec, e.b.var26);
        match boss {
            Some(b) => self.enemy_set_boss_hp(&mut e.b, b.hp_row),
            None => {
                e.b.hp = HP;
                e.b.hp_bar = HP;
                e.b.hp_prev = HP;
            }
        }
        e.b.stun = 10;
        e.b.reaction = 0;
        // Morgan's 0xFE keeps the controller from ending at his death (`boss_is_defeated` reads 0xFF).
        let att = match boss {
            Some(b) if b.rebirth.is_some() => 0xFE,
            Some(_) => 0xFF,
            None if spec.counted => 0,
            None => 0xFF,
        };
        e.m.sb(ATTACKING, att);
        if let Some(b) = boss.filter(|_| first) {
            // The boss init (0x4286A): its controller (`+$AA` = the boss) and the HUD bar (`G.-$70BC`).
            self.hud.boss = h;
            if b.controller {
                self.boss_spawn_controller(h);
            }
        }
        e.m.sb(WAIT, 0);
        e.m.sw(T80, 0);
        e.m.sb(SWING, 0);
        if spec.hazard {
            // Crosses from the side it starts on towards the far side of the screen.
            let on_left = e.b.xi() < self.cam.s2.cam_x as i16 + 0x100;
            e.m.sb(DIR, on_left as u8);
        }
        self.enemy_snap_to_floor(e);
        let t = self.enemy_pick_target(&e.b);
        e.m.sw(TARGET, t);
    }

    /// Morgan's death (0x52EBE): the same object becomes class `to` (Morgue, class 40) and its init runs. The fight goes on.
    fn enemy_ph_rebirth(&mut self, e: &mut Enemy, h: Handle, to: u16) {
        e.b.k20 = to * 4;
        e.b.sub = 0;
        e.b.hitstop = 0;
        let Some(spec) = spec_of(to) else { return };
        self.enemy_ph_setup(e, h, spec, false);
        e.b.mode = 2;
    }

    /// Mode 2. Returns true when the object was freed.
    fn enemy_ph_alive(&mut self, e: &mut Enemy, h: Handle, i: usize, spec: &Spec) -> bool {
        if boss_of(spec, e.b.var26) {
            // A placeholder boss has no intro: the boss area script waits for `G.$4FC` only while the intro runs.
            self.spawn.g.b4fc = 0;
        }
        // Dinosaurs leave when the camera reaches their leave X (+$5C) (D3b s1.3).
        let leave = e.m.w(W5C);
        if spec.dino && leave != 0 && self.cam.s2.cam_x >= leave {
            self.enemy_free_pool_a(e, i);
            return true;
        }
        if !boss_of(spec, e.b.var26) {
            self.enemy_kill_all_check(e);
        } else if self.spawn.g.kill_all == 2 && e.b.hp != 0 {
            // A boss ignores the kill-all of its own defeat (G.$50E = 1, 0xACEE). The review kill-all (2: `debug.rs`,
            // `--autoplay`) kills it through the boss hit path, so the defeat chain runs as after a real last hit.
            self.enemy_kill_all_check(e);
            self.boss_hit(e, h);
        }
        if spec.hazard {
            if self.enemy_ph_cross(e, i) {
                return true;
            }
        } else {
            match e.b.sub {
                0 if !spec.still => {
                    // A boss swings with attack 4 of the player table (12 damage), a walker with attack 0 (8).
                    let atk = if boss_of(spec, e.b.var26) { 4 } else { 0 };
                    self.enemy_ph_walk(e, atk);
                }
                2 => {
                    e.m.sb(SWING, 0);
                    self.enemy_hit_reaction(e, h);
                }
                6 => self.enemy_grabbed(e, h),
                _ => {}
            }
            self.enemy_clamp_to_playfield(e, i);
        }
        self.enemy_draw_check(&mut e.b, h);
        self.enemy_push_hurtable(h);
        false
    }

    /// Walker AI: walk to the nearest hero, swing when in reach (cooldown `COOLDOWN`).
    fn enemy_ph_walk(&mut self, e: &mut Enemy, atk: u8) {
        let cool = e.m.b(WAIT);
        if cool != 0 {
            e.m.sb(WAIT, cool - 1);
        }
        if e.m.b(SWING) != 0 {
            if self.enemy_step(&mut e.b) {
                e.m.sb(SWING, 0);
                self.enemy_anim(&mut e.b, 0);
            }
            return;
        }
        let Some(t) = self.enemy_ph_nearest(&e.b) else { return };
        e.m.sw(TARGET, t);
        let tb = self.body(t).clone();
        let dx = tb.x.wrapping_sub(e.b.x);
        let dz = tb.z.wrapping_sub(e.b.z);
        if dx.unsigned_abs() < REACH_X && dz.unsigned_abs() < REACH_Z && e.m.b(WAIT) == 0 {
            // The boxes and the punch of class 2 (anim 3); the damage record is `atk`.
            e.m.sb(WAIT, COOLDOWN);
            e.m.sb(SWING, 1);
            self.enemy_begin_attack(&mut e.b, atk);
            self.enemy_anim(&mut e.b, 3);
            return;
        }
        e.b.x = e.b.x.wrapping_add(dx.clamp(-STEP, STEP));
        e.b.z = e.b.z.wrapping_add(dz.clamp(-STEP, STEP));
        e.b.facing = (tb.x > e.b.x) as u8;
        self.enemy_floor_collision(e);
    }

    /// Hazard: crosses the screen at 1.5 px per frame. Returns true when it left the screen.
    fn enemy_ph_cross(&mut self, e: &mut Enemy, i: usize) -> bool {
        let right = e.m.b(DIR) != 0;
        e.b.x = e.b.x.wrapping_add(if right { STEP } else { -STEP });
        let dx = e.b.xi().wrapping_sub(self.cam.s2.cam_x as i16);
        if (right && dx > 0x240) || (!right && dx < -0x40) {
            self.enemy_free_pool_a(e, i);
            return true;
        }
        false
    }

    /// The living hero nearest to `b` (manhattan distance on the 16.16 positions).
    fn enemy_ph_nearest(&self, b: &Body) -> Option<Handle> {
        (0..3)
            .filter(|&p| self.players[p].b.live != 0)
            .min_by_key(|&p| Game::enemy_manhattan(b, &self.players[p].b))
            .map(Game::player_handle)
    }

    /// Mode 4: kill score (when the class has a name), then the blink, then free (mode 6).
    fn enemy_ph_dying(&mut self, e: &mut Enemy, h: Handle, spec: &Spec) {
        let boss = boss_part(spec, e.b.var26);
        if e.b.sub == 0 {
            if (spec.name.is_some() || spec.name_v2.is_some()) && boss.is_none_or(|b| b.score) {
                let b = e.b.clone();
                self.enemy_award_kill_score(&b);
            }
            e.m.sw(T80, BLINK);
            e.b.sub += 2;
            if boss.is_some_and(|b| b.rebirth.is_none()) {
                self.hook_a5ea(); // the first step of a boss death state fades the music (Butcher 0x38D44)
            }
        }
        match boss {
            // Morgan: after the blink he is Morgue, the same object (0x52EBE, the fight goes on).
            Some(Boss { rebirth: Some(to), .. }) => {
                let t = e.m.w(T80);
                if t == 0 {
                    self.enemy_ph_rebirth(e, h, to);
                } else {
                    e.m.sw(T80, t - 1);
                }
            }
            // a boss stays in its death state (the body stays, the controller and G.$4E0 read it)
            Some(_) => {}
            None => self.enemy_dying_blink(e, h),
        }
    }

    /// Whether the death of the boss `e` ends the fight (0xACEE): not Morgan (he becomes Morgue), and a Slisaur only
    /// when no other one of its class is alive. Any other object counts as a boss that ends it.
    pub fn boss_ends_fight(&self, e: &Enemy, h: Handle) -> bool {
        let Some(b) = spec_of(e.b.k20 / 4).and_then(|s| boss_part(s, e.b.var26)) else { return true };
        if b.rebirth.is_some() {
            return false;
        }
        !b.group
            || !self.enemies.slots.iter().enumerate().any(|(i, o)| {
                o.b.live != 0 && o.b.k20 == e.b.k20 && o.b.hp != 0 && self.enemies.handle(i) != h
            })
    }

    /// The starters of the boss classes 4 and 71 (pool B 79 and 33), placeholders too: they make their bosses at once and
    /// are done (the original waits for the scene first, not ported). Slisaur starter (0x818E4): two Slisaurs at X 0x580,
    /// Z 0x30 and 0x40, a third at Z 0x20 with two or three players, and the boss controller (0x7BEDC, `+$AA` = the starter).
    /// Tyrog starter (B79): Tyrog (class 4, form 0) where the starter stands.
    pub fn ph_starter(&mut self, p: &mut Prop, h: Handle, i: usize) {
        if self.trace_hold(h, &p.b) {
            return; // harness: kept as the snapshot had it
        }
        match p.b.k20 / 4 {
            33 => {
                let zs: &[u16] = if self.w.players_minus_one == 0 { &[0x30, 0x40] } else { &[0x30, 0x40, 0x20] };
                for &z in zs {
                    self.ph_spawn_boss(71, 0x580, z);
                }
                self.boss_spawn_controller(h);
            }
            _ => {
                let (x, z) = (p.b.xi() as u16, p.b.zi() as u16);
                self.ph_spawn_boss(4, x, z);
            }
        }
        self.prop_free(p, i);
    }

    /// Makes a pool A object of `class` (a boss record, variant 0) at X, Z: the next enemy update runs its init.
    fn ph_spawn_boss(&mut self, class: u16, x: u16, z: u16) {
        let r = SpawnRec { trig: 0, flags: 0, class: class * 4, x, z, var: 0, wc: 0, b5b: 0, b96: 0 };
        self.spawn_record(&r, false);
    }

    /// Placeholders are drawn as boxes (`draw_boxes`), not with their sprites: drop them from the
    /// sprite draw list. Harness runs (`hold_unported`) keep the list as the snapshot had it.
    pub fn enemy_ph_hide_sprites(&mut self) {
        if self.hold_unported {
            return;
        }
        let mut hide = Vec::new();
        for i in 0..self.enemies.slots.len() {
            let e = &self.enemies.slots[i];
            if e.b.live != 0 && spec_of(e.b.k20 / 4).is_some() && !Game::enemy_vice_intro_active(&e.b) {
                hide.push(self.enemies.handle(i) as u16);
            }
        }
        if !hide.is_empty() {
            self.draw_list.retain(|&k| !hide.contains(&(k as u16)));
        }
    }
}

/// 3 x 5 pixel digits (one row per byte, the low 3 bits are the pixels, bit 2 leftmost).
const DIGITS: [[u8; 5]; 10] = [
    [7, 5, 5, 5, 7],
    [2, 6, 2, 2, 7],
    [7, 1, 7, 4, 7],
    [7, 1, 7, 1, 7],
    [5, 5, 7, 1, 1],
    [7, 4, 7, 1, 7],
    [7, 4, 7, 5, 7],
    [7, 1, 1, 1, 1],
    [7, 5, 7, 5, 7],
    [7, 5, 7, 1, 7],
];

/// Draws every visible placeholder as a box with its class number (screen pixels), standing on
/// the object's feet like the sprites and the F1 boxes (`draw.rs` `debug_body`).
pub fn draw_boxes(g: &Game, gpu: &mut impl DrawTarget) {
    let (cx, cy) = (g.cam.s2.cam_x as i16, g.cam.s2.cam_y as i16);
    let fill = Color::new(0.95, 0.75, 0.2, 0.9);
    for e in &g.enemies.slots {
        let b = &e.b;
        if b.live == 0 || b.visible == 0 || spec_of(b.k20 / 4).is_none() || Game::enemy_vice_intro_active(b) {
            continue;
        }
        let x0 = b.xi().wrapping_sub(cx) as i32 - 0x40 - 12;
        let feet = super::draw::screen_y(b.yi().wrapping_add(b.zi()).wrapping_sub(cy)) as i32 - 16;
        let y0 = feet - 40;
        gpu.debug_rect(x0 as f32, y0 as f32, 24.0, 40.0, fill);
        let digits: Vec<usize> = (b.k20 / 4).to_string().bytes().map(|d| (d - b'0') as usize).collect();
        for (k, d) in digits.iter().enumerate() {
            for (row, bits) in DIGITS[*d].iter().enumerate() {
                for col in 0..3 {
                    if bits & (4 >> col) != 0 {
                        let px = x0 + 2 + 6 * k as i32 + 2 * col;
                        let py = y0 + 4 + 2 * row as i32;
                        gpu.debug_rect(px as f32, py as f32, 2.0, 2.0, BLACK);
                    }
                }
            }
        }
    }
}
