//! Placeholders for pool A classes the port does not run yet (M3-01). They keep the wave, lock,
//! GO arrow, clock and drop flow alive until the real class is ported. They are not exact:
//! a walker walks to the nearest hero and swings at him, a hazard crosses the screen, and every
//! placeholder is drawn as a labelled box (`draw_boxes`, called by `present.rs`).
//!
//! The registry `SPECS` lists the classes served by placeholders. A real port deletes its entry
//! (`--list-placeholders` prints the list). The census flag, the name ids and the kinds come from
//! `plan/game/design/D3a-humans.md` s1 and s13, `D3b-dinosaurs.md` s1 and s9, and the name ids of
//! `re/specs/enemy/classmap.txt` (`move.b #id,$3(a6)`). Owner: enemy port.

use super::body::Body;
use super::enemy::{Enemy, off::*};
use super::{Game, Handle};
use crate::render::DrawTarget;
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
    /// Boss variant (variant 0 only, `boss_of`): a stage boss, not counted, with the boss controller and the death chain.
    pub boss: bool,
}

const fn walker(class: u16, what: &'static str, name: Option<u8>, counted: bool) -> Spec {
    Spec { class, what, name, name_v2: None, counted, dino: false, hazard: false, still: false, boss: false }
}

const fn actor(class: u16, what: &'static str, counted: bool) -> Spec {
    Spec { class, what, name: None, name_v2: None, counted, dino: false, hazard: false, still: true, boss: false }
}

const fn dino(class: u16, what: &'static str, name: Option<u8>) -> Spec {
    Spec { class, what, name, name_v2: None, counted: false, dino: true, hazard: false, still: false, boss: false }
}

const fn cross(class: u16, what: &'static str, counted: bool) -> Spec {
    Spec { class, what, name: None, name_v2: None, counted, dino: false, hazard: true, still: false, boss: false }
}

/// Every pool A class of `stage_scripts.txt` that the port does not run yet.
pub const SPECS: &[Spec] = &[
    walker(3, "LASH T. (mace)", Some(0x0E), true),
    walker(5, "BUTCHER trigger", None, true),
    walker(6, "BLUDGE", Some(0x10), true),
    walker(13, "POACHER", Some(0x02), true),
    actor(16, "CADILLAC (blue)", true),
    Spec { class: 18, what: "VICE T. / WALTHER", name: Some(0x0C), name_v2: Some(0x0D), counted: true, dino: false, hazard: false, still: false, boss: true },
    dino(22, "BRONTOSAURUS foot", None),
    dino(24, "SHIVAT", Some(0x07)),
    walker(25, "SLICE", Some(0x05), true),
    actor(27, "OPENING man", false),
    dino(29, "R HOPPER", Some(0x04)),
    walker(31, "HOGG", Some(0x26), true),
    walker(35, "MORGAN intro", None, true),
    dino(38, "MACK", Some(0x11)),
    actor(43, "OPENING leader", false),
    walker(45, "MORGAN villager", None, true),
    walker(63, "BLADE alt", Some(0x1C), true),
    walker(64, "PUNK alt (bomb)", Some(0x1D), true),
    cross(65, "BARREL", false),
    cross(66, "ROCK", false),
    cross(70, "BIKER", true),
    cross(78, "CRITTER", false),
    walker(79, "HUNTER", Some(0x1E), true),
    cross(80, "CRITTER", false),
    dino(82, "ZEKE", Some(0x21)),
    walker(94, "FESSENDEN form 2", Some(0x25), true),
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
    spec.boss && (var26 >> 8) == 0
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
        let name = if (e.b.var26 >> 8) == 2 { spec.name_v2.or(spec.name) } else { spec.name };
        e.b.id = name.unwrap_or(0);
        // +$84 (per-type rows of the hit, grab and damage tables) stays at FERRIS's row 0: the grab
        // and damage rows of the other names are not extracted yet. The name (+$3) gives the score.
        e.m.sw(W84, 0);
        e.b.anim_table = ANIM_TABLE;
        e.b.box_table = BOX_TABLE;
        e.b.multi_box = false;
        self.enemy_anim(&mut e.b, 0);
        let boss = boss_of(spec, e.b.var26);
        if boss {
            self.enemy_set_boss_hp(&mut e.b, 0); // the Vice row of 0x31E14
        } else {
            e.b.hp = HP;
            e.b.hp_bar = HP;
            e.b.hp_prev = HP;
        }
        e.b.stun = 10;
        e.b.reaction = 0;
        e.m.sb(ATTACKING, if spec.counted && !boss { 0 } else { 0xFF });
        if boss {
            // The boss init (0x4286A): its controller (`+$AA` = the boss) and the HUD bar (`G.-$70BC`).
            self.hud.boss = h;
            self.boss_spawn_controller(h);
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
        e.b.mode += 2;
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
                0 if !spec.still => self.enemy_ph_walk(e),
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
    fn enemy_ph_walk(&mut self, e: &mut Enemy) {
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
            // Attack 0 of class 2 (its punch record and boxes): the swing is the thug's punch.
            e.m.sb(WAIT, COOLDOWN);
            e.m.sb(SWING, 1);
            self.enemy_begin_attack(&mut e.b, 0);
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
        if e.b.sub == 0 {
            if spec.name.is_some() || spec.name_v2.is_some() {
                let b = e.b.clone();
                self.enemy_award_kill_score(&b);
            }
            e.m.sw(T80, BLINK);
            e.b.sub += 2;
            if boss_of(spec, e.b.var26) {
                self.hook_a5ea(); // the first step of a boss death state fades the music (Butcher 0x38D44)
            }
        }
        if boss_of(spec, e.b.var26) {
            return; // a boss stays in its death state (the body stays, the controller and G.$4E0 read it)
        }
        self.enemy_dying_blink(e, h);
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
            if e.b.live != 0 && spec_of(e.b.k20 / 4).is_some() {
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
        if b.live == 0 || b.visible == 0 || spec_of(b.k20 / 4).is_none() {
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
