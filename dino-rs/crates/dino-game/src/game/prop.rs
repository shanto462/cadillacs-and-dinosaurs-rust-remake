//! Pool B: controllers and props (0x23604): the wave controller (class 15, 0x7B336) and the boss
//! reinforcement controller (class 22, 0x7BEF6). Owner: enemy port. See SPEC section 8.2.
//!
//! # Props the players can hit (T-B0: the prop contract, the hit list and the hit pass)
//!
//! A breakable prop (steam pipes, wall panels, the hall window, the fence, the flame vents) follows one contract, and
//! the collision pass `pass_10cd0` (0x10CD0) does the rest:
//!
//! - Init (mode 0): `Prop::hit_init(box_table, hp)` = body box index `+$48 = 1`, no attack box `+$49 = 0`, the box
//!   table `+$44` and the HP `+$6C` and `+$6E`. The prop box tables are in program ROM (0x8380A, 0x76666, 0x750B4 ...):
//!   the class adds its table (12 bytes a box) to `TABLES` of `dino-extract/src/enemy.rs`, and the pass reads the box
//!   from `assets.enemy.t` (the hit box bank only holds 0x100000 to 0x11D220).
//! - Every frame it can be hit (mode 2): `Game::prop_register_hit(handle)` = 0x4BFA, the list 0x6724 with the count
//!   0x671C. The list is emptied every frame (0x105E8): a prop that stops registering cannot be hit any more.
//! - The pass: for every player attack box of the list 0x6718 whose frame class `+$2E` is 4 or 6, and every
//!   registered prop that is not in a reaction (`+$00` bit 1 clear), when the swing is new (the serials `+$5E` differ)
//!   and the box test 0x128F8 hits (attack box `+$49` against body box `+$48`), the hit is taken: serial, player
//!   `+$02`, the state long `+$04 = 0x04000000` (mode 4, sub 0), `+$7A = 0`, the attacker `+$68`, HP `+$6E = +$6C`
//!   and `+$6C` minus the damage of the player's attack record (floor 0), hit stop `+$59` on both, the hit sound
//!   0x962A, the score 0xFFC8, `+$9A` and the hit spark 0x12720. **The pass never looks at the HP: one hit that
//!   overlaps the box always breaks the prop** (only the flame vents, class 45, count their HP themselves).
//! - The reach rule is the ordinary box test 0x128F8 (`collide::boxes_hit`) of the attack frame against the prop's
//!   body box, nothing special for props. D3e part 3 section 2.6 sums it up for the common jab and kick boxes (about
//!   8 deep): a hero at depth Z hits a prop whose box depth range is `[start, end]` when `Z + 8 >= start` and
//!   `Z <= end`, and the X and height ranges overlap. The mine wall (Z 0x60, range 32 to 128) is hit from Z 32 and not
//!   from Z 16, a vent whose range starts at Z 832 from Z 824 and not from Z 816 (the tests of this file). M3-16
//!   leaves this as the box test: the jab's `Z + 8` is that summary, not a second rule (M2-05 finished it).
//! - Mode 4 (the break) is the prop's own: tiles (`tile_block.rs`), sound (`Game::snd_break`), effect, then free.
//!   The palette cycler, the tile clock and the door burst shared by the later classes are in `drivers.rs` (M3-16).

use super::body::Body;
use super::collide;
use super::enemy::Ram;
use super::spawn::clear_header;
use super::{slot_of, Game, Handle, Slot};
use dino_assets::enemy::Wave;
use dino_assets::HitBox;

// The tile block writer and the block table driver of the scenery props (T-B0, M2-05 / M2-06).
#[path = "prop/tile_block.rs"]
pub mod tile_block;
// Palette cycler, tile clock and door burst (T3-0, M3-16).
#[path = "prop/drivers.rs"]
pub mod drivers;

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
// Pool B class 26 (M2-26): the typewriter of the episode title (0x7EC48), its text in `flow/text.rs`.
#[path = "prop/class26.rs"]
mod class26;
#[path = "prop/class66.rs"]
mod class66;
#[path = "prop/class72.rs"]
mod class72;
// Pool B class 25 (M2-24): the numbers of the stage clear tally.
#[path = "prop/b25.rs"]
mod b25;
// Pool B class 74 (M2-11): the master of the Episode 1 opening scene (it makes the pool A cast 54, 55, 56).
#[path = "prop/b74.rs"]
mod b74;
// Episode 1 transition props (M3-17): the metal door (11), the hole and wooden door (14), the tile change (21).
#[path = "prop/b11.rs"]
mod b11;
#[path = "prop/b14.rs"]
mod b14;
#[path = "prop/b21.rs"]
mod b21;
// Episode 1 hall door and window (M3-18, class 12, 0x78B52).
#[path = "prop/b12.rs"]
mod b12;
// Pool B class 31 (M2-13): the stage clock (it keeps its sprite list in a pool D slot).
#[path = "prop/b20.rs"]
mod b20;
#[path = "prop/b31.rs"]
mod b31;
#[path = "prop/b35.rs"]
mod b35;
#[path = "prop/b47.rs"]
mod b47;
// Episode 8 sewer end (M3-22): the waterfall over the edge and the shaft (classes 36, 37, 38).
#[path = "prop/b36_38.rs"]
mod b36_38;
// Pool B class 18 (M2-16): the respawn air strike (five bombs from the record list 0x7B7A6).
#[path = "prop/b18.rs"]
mod b18;
// Pool B classes of the character select (M2-04): the frame (17), the pictures over the group picture (67) and the player tags (68).
#[path = "prop/class17.rs"]
mod class17;
#[path = "prop/class19.rs"]
mod class19;
#[path = "prop/class67.rs"]
mod class67;
#[path = "prop/class68.rs"]
mod class68;
// Pool B class 96 (M3-15): area music object after sub-area camera mode 8.
#[path = "prop/class96.rs"]
mod class96;
// The ending (M3-23): the master of the scenes (82) and the showers and quakes (83, 84, 85).
#[path = "prop/b82_85.rs"]
mod b82_85;
// Pool B class 1 (M3-19): chain-link fence / gate (0x75238).
#[path = "prop/b1.rs"]
mod b1;
// Pool B class 3 (M3-19): mine wall (0x763F6).
#[path = "prop/b3.rs"]
mod b3;

#[derive(Clone, Debug, Default)]
pub struct Prop {
    pub b: Body,
    pub m: Ram,
}

impl Prop {
    /// The prop contract of a breakable prop (class 40 init 0x836B8, class 12 var 4 init 0x78FC0): body box index
    /// `+$48 = 1`, attack box `+$49 = 0`, box table `+$44`, HP `+$6C = +$6E`. The hit pass 0x10CD0 never reads the HP.
    #[allow(dead_code)] // used by the breakable props of M3
    pub fn hit_init(&mut self, box_table: u32, hp: u16) {
        self.b.body_box = 1;
        self.b.attack_box = 0;
        self.b.box_table = box_table;
        self.b.hp = hp;
        self.b.hp_prev = hp;
    }
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
                11 => self.prop_b11(&mut p, i),
                12 => self.prop_b12(&mut p, i),
                14 => self.prop_b14(&mut p, i),
                1 => self.prop_b1(&mut p, h, i),
                3 => self.prop_b3(&mut p, h, i),
                15 => self.prop_wave(&mut p, h, i),
                22 => self.boss_controller(&mut p, i),
                17 => self.prop_class17(&mut p, i),
                18 => self.prop_b18(&mut p, h, i),
                19 => self.prop_class19(&mut p, i),
                20 => self.prop_go_hint(&mut p, i),
                25 => self.prop_class25(&mut p, i),
                26 => self.prop_class26(&mut p, i),
                21 => self.prop_b21(&mut p, i),
                31 => self.prop_clock(&mut p, i),
                32 => self.prop_class32(&mut p, i),
                35 => self.prop_b35(&mut p, i),
                36 => self.prop_b36(&mut p, i),
                37 => self.prop_b37(&mut p, i),
                38 => self.prop_b38(&mut p, i),
                47 => self.prop_b47(&mut p, i),
                66 => self.prop_class66(&mut p, i),
                67 => self.prop_class67(&mut p, i),
                68 => self.prop_class68(&mut p, i),
                69 => self.prop_debug_object(&mut p, i),
                72 => self.prop_class72(&mut p, i),
                73 => self.prop_bird_spawner(&mut p, i),
                74 => self.prop_class74(&mut p, i),
                80 => self.prop_ambience(&mut p, i),
                82 => self.prop_b82(&mut p, i),
                83 => self.prop_b83(&mut p, i),
                84 => self.prop_b84(&mut p, i),
                85 => self.prop_b85(&mut p, i),
                86 => self.prop_class86(&mut p, i),
                89 => self.prop_class89(&mut p, i),
                93 => self.prop_class93(&mut p, i),
                33 | 79 => self.ph_starter(&mut p, h, i), // placeholders of the boss starters (placeholder.rs)
                94 => self.prop_class94(&mut p, i),
                96 => self.prop_class96(&mut p, i),
                _ => self.prop_unported(&mut p, h, i),
            }
            self.props.slots[i] = p;
        }
    }

    /// 0x4BFA: the prop can be hit by the players this frame (the list 0x6724 and its count 0x671C; the passes
    /// visit the newest entry first). Call it every frame from the prop's idle mode.
    #[allow(dead_code)] // used by the breakable props of M3
    pub fn prop_register_hit(&mut self, h: Handle) {
        self.combat.push(0x671C, h);
    }

    /// 0x10CD0: player attacks against the registered props (see the module doc). Called by `collision` (0x10546)
    /// after the pass against enemies, before the pass against containers.
    pub fn pass_10cd0(&mut self) {
        let props = self.combat.get(0x671C).to_vec(); // 0x10CE2: no props, nothing to do
        if props.is_empty() {
            return;
        }
        for a2 in self.combat.get(0x6718).to_vec() {
            let Some(Slot::Player(i)) = slot_of(a2) else { continue };
            // 0x10D16 by the frame class +$2E: only 4 and 6 attack; for the others 0x10D26 ends the prop loop (clr.w d7).
            if !matches!(self.players[i].b.kind, 4 | 6) {
                continue;
            }
            for &a3 in &props {
                self.prop_hit_by_player(i, a2, a3);
            }
        }
    }

    /// The body box of a prop: the record `+$48` of the box table `+$44`, read from the extracted byte tables
    /// (`dino-extract/src/enemy.rs` `TABLES`, 12 bytes a box). The prop tables are in program ROM (0x75414, 0x8380A,
    /// 0x76666 ...), outside the region of `assets.sprites.hitboxes` (0x100000 to 0x11D220), so each prop class adds its
    /// table to `TABLES`. Single box tables only (+$4A = 0), which is all the props use.
    fn prop_body_box(&self, b: &Body) -> HitBox {
        let a = (b.box_table & 0xFF_FFFF) + 12 * b.body_box as u32;
        let t = &self.assets.enemy.t;
        let w = |k: u32| t.i16(a, 2 * k);
        HitBox { zoff: w(0), zsize: w(1), xoff: w(2), xsize: w(3), yoff: w(4), ysize: w(5) }
    }

    /// 0x128F8 for a player's attack box (`collide::boxes`, the hit box bank) against the body box of a prop
    /// (`prop_body_box`): the same three unsigned range tests as `collide::boxes_hit`, whose `overlap` is private. Returns
    /// the first overlapping pair for the spark, as it does.
    fn prop_box_hit(&self, attacker: &Body, prop: &Body) -> Option<(HitBox, HitBox)> {
        fn axis(a0: i16, asz: i16, b0: i16, bsz: i16) -> bool {
            (a0.wrapping_sub(b0).wrapping_add(asz) as u16) <= (asz.wrapping_add(bsz) as u16)
        }
        let hb = self.prop_body_box(prop);
        collide::boxes(&self.assets.sprites, attacker, attacker.attack_box)
            .into_iter()
            .find(|ha| {
                axis(attacker.zi().wrapping_add(ha.zoff), ha.zsize, prop.zi().wrapping_add(hb.zoff), hb.zsize)
                    && axis(collide::left(attacker, ha), ha.xsize, collide::left(prop, &hb), hb.xsize)
                    && axis(attacker.yi().wrapping_add(ha.yoff), ha.ysize, prop.yi().wrapping_add(hb.yoff), hb.ysize)
            })
            .map(|ha| (ha, hb))
    }

    /// 0x10D2A: player `i` (object `a2`) against the prop `a3`.
    fn prop_hit_by_player(&mut self, i: usize, a2: Handle, a3: Handle) {
        let Some(Slot::Prop(pi)) = slot_of(a3) else { return };
        let ab = self.players[i].b.clone();
        let pb = &self.props.slots[pi].b;
        // 0x10CF2: a prop in a reaction (+$00 bit 1) is skipped; 0x10D2A: one swing hits it once; the two box indexes.
        if pb.live & 2 != 0 || ab.serial == pb.serial || ab.attack_box == 0 || pb.body_box == 0 {
            return;
        }
        let Some((ha, hb)) = self.prop_box_hit(&ab, pb) else {
            return;
        };
        // 0x10D78: the record of the player's attack, 8 bytes at 0xA26FC[character] + +$58 * 4.
        let ch = (ab.k20 / 4) as usize & 3;
        let r = self.assets.sprites.player_attacks[ch].get((ab.atk / 2) as usize).copied().unwrap_or_default();
        {
            let p = &mut self.props.slots[pi];
            p.b.serial = ab.serial;
            p.b.who = ab.who;
            p.b.set_state(0x0400_0000); // routine 4, sub 0: the break
            p.m.sb(0x7A, 0);
            p.b.attacker = a2;
            p.b.hp_prev = p.b.hp;
            let hp = (p.b.hp as i16).wrapping_sub(r.damage as i16);
            p.b.hp = if hp > 0 { hp as u16 } else { 0 }; // bgt / clr.w: the floor is 0
            p.b.hitstop = r.hit_stop; // 0x10DA2: +$59 of both
            p.b.b9a = r.b7;
        }
        self.players[i].b.hitstop = r.hit_stop;
        if r.sound != 0 {
            self.snd_hit(r.sound); // 0x962A
        }
        self.player_add_score(i, r.score); // 0xFFC8
        self.spawn_hit_spark(a2, a3, ha, hb, r.spark); // 0x12720 (bra)
    }

    /// Classes not ported yet (props, scenery, event objects): freed at once.
    fn prop_unported(&mut self, p: &mut Prop, h: Handle, i: usize) {
        if self.trace_hold(h, &p.b) {
            return; // harness: kept as the snapshot had it
        }
        self.prop_free(p, i);
    }

    /// 0x8A582 pool B class 80 (`+$20 = 0x140`): the ambience of the stage 1 (Episode 2), made by its start script. Mode 0 (0x8A596) goes to mode 2 and sets the
    /// counter `+$A0` to 0x78; mode 2 counts it down and every 0x78 frames draws a number: below 4 of 16 a second one decides between the sound 0x78 and nothing
    /// (the other branch, 0x8A5D0, loads the sound 0x61 into d0 and returns without the request). Mode 4 frees it.
    fn prop_ambience(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                p.b.mode = 2;
                p.m.sw(0xA0, 0x78);
            }
            2 => {}
            _ => return self.prop_free(p, i), // 0x8A5F4
        }
        let left = p.m.w(0xA0).wrapping_sub(1);
        p.m.sw(0xA0, left);
        if left != 0 {
            return;
        }
        p.m.sw(0xA0, 0x78);
        if self.rand() & 0xF < 4 && self.rand() & 1 != 0 {
            self.snd_request(0x78, 0, 0);
        }
    }

    /// 0x88A84 pool B class 69: the object of the debug mode (the button code 0x88BBE, the ENEMY spawner 0x88DDA). The stage start makes it; its first update
    /// (0x88A98) frees it at once when the debug switch G.$4AA is off (0x88DC6). The debug mode itself is not ported (G.$4AA is not tracked: off).
    fn prop_debug_object(&mut self, p: &mut Prop, i: usize) {
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
                self.prop_set_leader(); // 0x7B4F2
                self.script_area_end(); // 0x1D95E: the area end script, the heroes walk to the next area
                p.b.mode = 6;
                p.b.sub = 0;
            }
            // end type 6 points into data at 0x7D054 (the table 0x7B4D8 entry); no working stage uses it
            _ => {
                self.prop_set_leader(); // 0x7B506
                self.script_boss_area(); // 0x1DAC4: the boss area script
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::{AssetPack, AttackRecord, HitBoxBank};

    /// Hit box tables (synthetic, 6 words a box, 12 bytes). The hero's attack box (in the hit box bank, as the real ones)
    /// is 8 deep like D3e's summary of the jab: depth 0 to 8, X 0x10 to 0x30, height 0x10 to 0x30. The prop tables are
    /// byte tables of `enemy.t`, as the prop classes extract them: the mine wall box is [Z - 64, Z + 32] (depth range
    /// 32 to 128 for Z 0x60), 0x20 wide from X, 0x80 high; the vent box starts at its Z and is 32 deep.
    const HERO: u32 = 0x100000;
    const WALL: u32 = 0x83A00;
    const VENT: u32 = 0x83A18;

    fn game() -> Game {
        let mut a = AssetPack::default();
        let hero: Vec<i16> = [[0, 0, 0, 0, 0, 0], [0, 8, 0x10, 0x20, 0x10, 0x20]].concat();
        a.sprites.hitboxes = HitBoxBank { base: HERO, words: hero };
        let bytes = |w: [i16; 12]| w.iter().flat_map(|v| v.to_be_bytes()).collect::<Vec<u8>>();
        a.enemy.t.blobs = vec![
            (WALL, bytes([0, 0, 0, 0, 0, 0, -64, 96, 0, 0x20, 0, 0x80])),
            (VENT, bytes([0, 0, 0, 0, 0, 0, 0, 32, 0, 0x20, 0, 0x80])),
        ];
        a.sprites.player_attacks[0] = vec![AttackRecord { damage: 3, stun: 1, reaction: 2, hit_stop: 9, sound: 0, score: 0, spark: 1, b7: 5 }; 4];
        Game::new(Box::leak(Box::new(a)))
    }

    /// One swing of hero 0 (depth `hero_z`, X 0x100, attack frame class 4, serial 5) against the prop in slot 23 with the
    /// box table `table` at depth `prop_z`; the prop is registered as its idle mode does. Returns the game.
    fn swing(table: u32, prop_z: i16, hero_z: i16) -> Game {
        let mut g = game();
        let me = Game::player_handle(0);
        {
            let p = &mut g.players[0].b;
            p.live = 1;
            p.box_table = HERO;
            p.attack_box = 1;
            p.kind = 4;
            p.serial = 5;
            p.set_xi(0x100);
            p.set_zi(hero_z);
        }
        let i = g.props.alloc().unwrap();
        let h = g.props.handle(i);
        {
            let p = &mut g.props.slots[i];
            p.b.live = 1;
            p.b.set_xi(0x120);
            p.b.set_zi(prop_z);
            p.hit_init(table, 8);
        }
        g.combat.push(0x6718, me);
        g.prop_register_hit(h);
        g.pass_10cd0();
        g
    }

    fn broken(g: &Game) -> bool {
        g.props.slots[23].b.state() == 0x0400_0000
    }

    /// D3e part 3 section 2.6, the mine wall (Z 0x60, box depth 32 to 128): a hero at depth 32 hits, one at 16
    /// misses; the edges are `Z + 8 >= start` and `Z <= end`.
    #[test]
    fn the_reach_rule_of_the_mine_wall() {
        assert!(broken(&swing(WALL, 0x60, 32)));
        assert!(!broken(&swing(WALL, 0x60, 16)));
        assert!(broken(&swing(WALL, 0x60, 79)));
        assert!(broken(&swing(WALL, 0x60, 24)), "24 + 8 reaches 32");
        assert!(!broken(&swing(WALL, 0x60, 23)));
        assert!(broken(&swing(WALL, 0x60, 128)), "the end is included");
        assert!(!broken(&swing(WALL, 0x60, 129)));
    }

    /// The flame vents (B45, box depth from 832): a hero at the back limit side 824 hits, 816 misses.
    #[test]
    fn the_reach_rule_of_the_vents() {
        assert!(broken(&swing(VENT, 832, 824)));
        assert!(broken(&swing(VENT, 832, 831)));
        assert!(!broken(&swing(VENT, 832, 816)));
    }

    /// The hit itself (0x10D2A): the state long, serial, player, attacker, HP floor 0 with the old HP kept, hit stop
    /// on both, +$9A, a spark; the prop never looks at its HP.
    #[test]
    fn a_hit_takes_the_prop_into_its_break() {
        let g = swing(WALL, 0x60, 40);
        let p = &g.props.slots[23].b;
        assert_eq!((p.state(), p.serial, p.who, p.attacker), (0x0400_0000, 5, 0, Game::player_handle(0)));
        assert_eq!((p.hp, p.hp_prev, p.hitstop, p.b9a), (5, 8, 9, 5));
        assert_eq!(g.players[0].b.hitstop, 9);
        assert_eq!(g.effects.free_count(), 47, "a spark");
        // A prop with 1 HP: the damage 3 takes it to the floor 0 (and it breaks like any other).
        let mut g = game();
        g.players[0].b = swing(WALL, 0x60, 40).players[0].b.clone();
        let i = g.props.alloc().unwrap();
        g.props.slots[i].b.live = 1;
        g.props.slots[i].b.set_xi(0x120);
        g.props.slots[i].b.set_zi(0x60);
        g.props.slots[i].hit_init(WALL, 1);
        g.combat.push(0x6718, Game::player_handle(0));
        g.prop_register_hit(g.props.handle(i));
        g.pass_10cd0();
        assert_eq!((g.props.slots[i].b.hp, g.props.slots[i].b.hp_prev), (0, 1));
    }

    /// One swing hits once; a new swing (a new serial) hits again; frame classes other than 4 and 6, a prop that is not
    /// registered, a prop in a reaction and a missing box index do nothing.
    #[test]
    fn the_rules_of_the_pass() {
        let mut g = swing(WALL, 0x60, 40);
        let h = g.props.handle(23);
        let reset = |g: &mut Game| {
            g.props.slots[23].b.set_state(0x0200_0000);
            g.props.slots[23].b.hp = 8;
            g.combat.clear(0x671C);
            g.prop_register_hit(h);
        };
        reset(&mut g);
        g.pass_10cd0();
        assert!(!broken(&g), "the same swing (serial 5) does not hit again");
        g.players[0].b.serial = 6;
        g.pass_10cd0();
        assert!(broken(&g), "a new swing does");
        for kind in [0, 2, 8, 0xA] {
            reset(&mut g);
            g.players[0].b.serial = 7;
            g.players[0].b.kind = kind;
            g.pass_10cd0();
            assert!(!broken(&g), "frame class {kind}");
        }
        g.players[0].b.kind = 6; // 6 attacks too
        reset(&mut g);
        g.pass_10cd0();
        assert!(broken(&g));
        reset(&mut g);
        g.players[0].b.serial = 8;
        g.combat.clear(0x671C);
        g.pass_10cd0();
        assert!(!broken(&g), "not registered this frame");
        g.prop_register_hit(h);
        g.props.slots[23].b.live = 2;
        g.pass_10cd0();
        assert!(!broken(&g), "in a reaction");
        g.props.slots[23].b.live = 1;
        g.props.slots[23].b.body_box = 0;
        g.pass_10cd0();
        assert!(!broken(&g), "no body box");
        g.props.slots[23].b.body_box = 1;
        g.players[0].b.attack_box = 0;
        g.pass_10cd0();
        assert!(!broken(&g), "no attack box");
        g.players[0].b.attack_box = 1;
        g.pass_10cd0();
        assert!(broken(&g));
    }

    /// The list is emptied every frame (0x105E8), so a prop that stops registering cannot be hit.
    #[test]
    fn the_hit_list_is_per_frame() {
        let mut g = game();
        g.prop_register_hit(g.props.handle(3));
        g.prop_register_hit(g.props.handle(5));
        assert_eq!(g.combat.get(0x671C), &[g.props.handle(5), g.props.handle(3)], "newest first");
        g.combat_lists_reset();
        assert!(g.combat.get(0x671C).is_empty());
    }
}
