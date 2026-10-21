//! Scripted control of the heroes (T-C1, M2-08 / M2-09): the interpreter that walks the heroes in at the start of an area, walks them
//! to the next one, runs the boss death scenes and the openings. Every area change, walk-in, walk-off and boss exit of the whole game
//! runs through it. Owner: flow port. Spec: `plan/game/design/D4c-camera-flow-transitions.md` section 5.3 to 5.6.
//!
//! The scripts (table 0x1E6FC, 26 scripts of six byte records) and the tables of targets are data and come from the asset pack
//! (`assets.script`, `dino-extract/src/script.rs`). The code of the 41 actions is ported here, each with its original address.
//!
//! # How it runs
//!
//! * Three entry points set the script id G.$4DE and the mask G.$4DD (bit i = hero i is under script): `script_area_start` (0x1DA10, called by
//!   the area start 0xAA0E), `script_area_end` (0x1D95E, wave end type 2 and the boss death) and `script_boss_area` (0x1DAC4, wave end type 4).
//!   None of them runs anything: they clear the `+$D0` of the heroes.
//! * The player update (0x18952, after the pad is read) calls, while G.$4DD is not 0, `script_hero_start` (0x1DB0C) once for a hero whose
//!   `+$D0` is 0 (it copies G.$4DD into it) and `script_step` (0x1DB5E) every frame.
//! * `script_step` runs only while the hero's state word `+$04` (mode and sub) is 0x0200, that is while he is alive and under his own control:
//!   every ordinary state (idle, walk, run, air, attacks, the poses and hops) counts, a hit reaction (sub 2 and more), a death (mode 4) or a
//!   respawn (mode 0) stops the script until it is over. His bit must be set in G.$4DD. The timer `+$D8` counts game frames: a record with the timer N runs its action in N frames. When it reaches 0 the next six
//!   byte record is read (action, G.$4F0, G.$4FD, G.$506, timer). An action with the timer 0xFFFF is a "wait until" loop that writes
//!   0xFFFF again every frame, or 1 when its condition is met. The actions overwrite the pad bytes `+$A0` (held) and `+$A8` (pressed):
//!   most of them end with 0x1DEA6 (`script_pad_clear`), which clears `+$A0`, `+$A8` and `+$BC` of all three heroes.
//! * Walking (0x1DF10, `script_walk_body`) keeps the direction of the last frame (`+$A0 = +$A1`) and recomputes it every fifth frame.
//!
//! # Fields
//!
//! The per hero fields `+$C0` (leader, `Player::bc0`), `+$D0` (copy of G.$4DD, `Player::bd0`) and `+$D1` to `+$DE` (`HeroScript`) live in
//! the player. G.$4DD is `World::scripted`, G.$4F0 `World::floor_only`, G.$506 `World::no_clamp`, G.$4DE `spawn.g.b4de`, G.$4FC
//! `spawn.g.b4fc`, G.$518 `spawn.g.car_mode`, G.$4E2 / G.$4E3 `flow.b4e2` / `b4e3`, G.$7A9 (the camera's go flag) `cam.s2.go`; the other
//! globals are in `ScriptGlobals` (`flow.script`).
//!
//! # Quirks kept
//!
//! * The walk compares X and Z as unsigned words (`bcs`): a hero at a negative X whose target is positive walks left.
//! * A non-leader hero whose two other end spots are both taken gets spot 0 (its distance starts at 0xFFFF and the others are 0xFFFF too).
//! * The action 0x26 (the end of the Episode 1 boss scene) does not clear G.$4DD, it clears only the bit of its own hero and the `+$D0` of
//!   all three: a hero with a later update in the same frame whose bit is still set starts his script again from the first record.
//! * G.$4F0 and G.$506 keep the values of the last record read, the END record's included, until the next script's first record.

use super::{ATK, DOWN, JMP, LEFT, RIGHT, UP};
use crate::game::Game;

/// The script fields of a hero (`+$D1` .. `+$DE`). The script pointer is kept as the ROM address of the next record.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HeroScript {
    /// `+$D1` script state (always 0: the table 0x1DB6C has one entry).
    pub state: u8,
    /// `+$D2` the current action (the byte offset into the table 0x1DBD6).
    pub act: u8,
    /// `+$D3` step inside the action: 0, then 2 after the first run.
    pub step: u8,
    /// `+$D4` pointer to the next record.
    pub ptr: u32,
    /// `+$D8` timer in game frames (0xFFFF = run the action again next frame).
    pub timer: u16,
    /// `+$DA` walk rank (the slot of the target, 0 = front spot).
    pub rank: u8,
    /// `+$DB` frames until the direction is recomputed (walks), or the frames since the jump (action 0x16).
    pub count: u8,
    /// `+$DC` target X and `+$DE` target Z of the walk.
    pub tx: u16,
    pub tz: u16,
}

/// The globals of the scripted control that no other module owns.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScriptGlobals {
    /// G.$4D8 set by the action 0x28 and cleared by the end (read by the skip wait task 0x9E38C, not ported).
    pub b4d8: u8,
    /// G.$4DF arrival bits of the walks (bit i = hero i is at his spot).
    pub b4df: u8,
    /// G.$4EC .. G.$4EE the end target slots that a hero has taken.
    pub taken: [u8; 3],
    /// G.$4FD the argument byte of the current record (caption number, sound code).
    pub b4fd: u8,
    /// G.$503 the pose lock (set by the action 0x04, cleared by the pose code).
    pub b503: u8,
    /// G.$514 set by the area end tables: 1 lets a hero keep a barrel through the walk (0x19090).
    pub b514: u8,
    /// G.$516 the clear sequence of Episode 8 is running: the player update skips the hero state machine (0x18A28).
    pub b516: u8,
    /// G.-$70AA the clear sequence has been started, G.-$70A9 the characters it has put on the stage (bits).
    pub b70aa: u8,
    pub b70a9: u8,
}

/// Which table the walk to targets uses (the actions 0x02, 0x0E, 0x1E and 0x14 share the walk body 0x1DF10).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Walk {
    /// 0x02: the end targets of the stage (0x1DFCA).
    End,
    /// 0x0E: the start targets of the stage and area (0x1E07E).
    Start,
    /// 0x1E: the lineup spots of the stage (0x1E13A).
    Lineup,
    /// 0x14: three fixed spots by the order of the heroes (0x1E4E4).
    Spots,
}

/// The table addresses of the original (the data is in `assets.script.tables`).
mod tab {
    /// 0x1D9B0 end ids: 8 pointers to (id, G.$514) per area.
    pub const END_IDS: u32 = 0x1D9B0;
    /// 0x1DA64 start ids: 8 pointers to a byte pair per area.
    pub const START_IDS: u32 = 0x1DA64;
    /// 0x1DAFC boss area ids: (id, G.$514) per stage.
    pub const BOSS_IDS: u32 = 0x1DAFC;
    /// 0x1E3F4 single points: X, Z per area, 16 bytes a stage.
    pub const POINTS: u32 = 0x1E3F4;
    /// 0x1E532 three fixed spots (the second bank for stage 7 is 12 bytes on).
    pub const SPOTS: u32 = 0x1E532;
    /// 0x1EE6A end targets: 16 bytes a stage (three spots and the cabinet row).
    pub const END_TARGETS: u32 = 0x1EE6A;
    /// 0x1EEEA start targets: 8 pointers to 16 bytes an area.
    pub const START_TARGETS: u32 = 0x1EEEA;
    /// 0x1F10A lineup spots: 16 bytes a stage.
    pub const LINEUP: u32 = 0x1F10A;
}

/// 0x1926: the distance from a hero (X, Z) to a spot, `|sx - x| + |sz - z|` on unsigned words (`sub.w` then `neg.w` on the carry), wrapping.
fn distance(x: u16, z: u16, sx: u16, sz: u16) -> u16 {
    let dx = if sx < x { x.wrapping_sub(sx) } else { sx.wrapping_sub(x) };
    let dz = if sz < z { z.wrapping_sub(sz) } else { sz.wrapping_sub(z) };
    dx.wrapping_add(dz)
}

impl Game {
    /// The part of the game task's sweep 0xA804 (G.$4CC .. G.$4EB) that holds scripted control state: G.$4D8 and G.$4DF.
    pub fn script_game_clear(&mut self) {
        self.flow.script.b4d8 = 0;
        self.flow.script.b4df = 0;
    }

    // ------------------------------------------------------------------------------------------------ entry points

    /// The start of 0x1DA10, 0x1D95E and 0x1DAC4: `+$D0` of the three heroes, G.$4DD and G.$4DE cleared.
    fn script_clear_all(&mut self) {
        for p in self.players.iter_mut() {
            p.bd0 = 0; // 0x3344, 0x34C4, 0x3644
        }
        self.w.scripted = 0;
        self.spawn.g.b4de = 0;
    }

    /// 0x1DA10 area start (called by 0xAA0E after the heroes are placed): the script of this stage and area (table 0x1DA64), all chosen heroes
    /// (G.$4E4) under script. Except in Episode 1 area 2 the leader flags are cleared (that area starts with the leader of the area before).
    pub fn script_area_start(&mut self) {
        self.script_clear_all();
        let t = &self.assets.script.tables;
        let (stage, area) = (self.w.stage as u32, self.cam.s2.area as u32);
        let row = t.u32(tab::START_IDS, 4 * stage); // movea.l 0x1DA64(pc,d0.w),a0
        self.spawn.g.b4de = t.u8(row, 2 * area); // move.b (a0)+,G.$4DE
        self.w.scripted = self.flow.chosen; // G.$4E4
        if self.w.stage == 0 && self.cam.s2.area == 2 {
            return;
        }
        for p in self.players.iter_mut() {
            p.bc0 = 0; // 0x3334, 0x34B4, 0x3634
        }
    }

    /// 0x1D95E area end (wave end type 2 and the boss death): the script by stage and area (table 0x1D9B0) and G.$514. The caller has set the
    /// leader (0x1E5F6). Script id 5 (Episode 1 area 1, the window) clears the camera's go flag G.$7A9.
    pub fn script_area_end(&mut self) {
        self.script_clear_all();
        let t = &self.assets.script.tables;
        let (stage, area) = (self.w.stage as u32, self.cam.s2.area as u32);
        let row = t.u32(tab::END_IDS, 4 * stage);
        self.spawn.g.b4de = t.u8(row, 2 * area);
        self.flow.script.b514 = t.u8(row, 2 * area + 1);
        if self.spawn.g.b4de == 5 {
            self.cam.s2.go = 0; // clr.b $7A9
        }
        self.w.scripted = self.flow.chosen;
        self.flow.script.b70aa = 0;
        self.flow.script.b70a9 = 0;
    }

    /// 0x1DAC4 boss area (wave end type 4): G.$4FC = 1 (the boss intro is running), the script by stage (table 0x1DAFC) and G.$514.
    pub fn script_boss_area(&mut self) {
        self.script_clear_all();
        self.spawn.g.b4fc = 1;
        let t = &self.assets.script.tables;
        let stage = self.w.stage as u32;
        self.spawn.g.b4de = t.u8(tab::BOSS_IDS, 2 * stage);
        self.flow.script.b514 = t.u8(tab::BOSS_IDS, 2 * stage + 1);
        self.w.scripted = self.flow.chosen;
    }

    // ------------------------------------------------------------------------------------------------ the step

    /// 0x189CE .. 0x189EE in the player update: the script of hero `i`, while G.$4DD is not 0. A hero whose `+$D0` is 0 copies G.$4DD there and
    /// starts (0x1DB0C), then the step runs (0x1DB5E).
    pub fn script_player_hook(&mut self, i: usize) {
        if self.w.scripted == 0 {
            return;
        }
        if self.players[i].bd0 == 0 {
            self.players[i].bd0 = self.w.scripted;
            self.script_hero_start(i);
        }
        self.script_step(i);
    }

    /// 0x1DB0C: the script of a hero starts. Position at the first record (timer 1, so the first step reads it), the end target slots free.
    pub fn script_hero_start(&mut self, i: usize) {
        for t in self.flow.script.taken.iter_mut() {
            *t = 0; // clr.b $4EC, $4ED, $4EE
        }
        let id = self.spawn.g.b4de as usize;
        let ptr = self.assets.script.pointers[id]; // table 0x1E6FC, the long at G.$4DE * 4
        let s = &mut self.players[i].sc;
        s.state = 0;
        s.act = 0;
        s.step = 0;
        s.timer = 1;
        s.ptr = ptr;
    }

    /// 0x1DB5E: one frame of the script of hero `i`. Only while his state word (mode, sub) is 0x0200 and his bit is set in G.$4DD.
    pub fn script_step(&mut self, i: usize) {
        let p = &self.players[i];
        if p.b.mode != 2 || p.b.sub != 0 {
            return; // cmpi.w #$200,$4(a6); bne rts
        }
        let bit = 1u8 << (p.b.who & 7);
        if self.w.scripted & bit == 0 {
            return; // btst d1,G.$4DD
        }
        let s = &mut self.players[i].sc;
        s.timer = s.timer.wrapping_sub(1);
        if s.timer == 0 {
            // 0x1DB88: the next record
            let r = self.assets.script.record_at(s.ptr);
            s.step = 0;
            s.ptr += 6;
            s.act = r.action;
            s.timer = r.timer;
            if matches!(r.action, 0x0A | 0x26) {
                self.w.scripted &= !bit; // both ends take the bit of this hero out of G.$4DD
            }
            self.w.floor_only = r.floor_only; // G.$4F0
            self.flow.script.b4fd = r.argument; // G.$4FD
            self.w.no_clamp = r.control; // G.$506
        }
        self.script_action(i);
    }

    /// 0x1DBC6: run the action of the record (jump table 0x1DBD6, the id is the byte offset).
    fn script_action(&mut self, i: usize) {
        match self.players[i].sc.act {
            0x00 => self.script_pad_clear(), // 0x1DEE0
            0x02 => self.script_walk(i, Walk::End), // 0x1DEF8
            0x04 => self.script_pose(i), // 0x1E200
            0x06 => self.script_scene_done(i), // 0x1E262
            0x08 => self.script_leader_attack(i), // 0x1E28C
            0x0C => self.script_special(i), // 0x1E2EE
            0x0A => {
                // 0x1E2BA: free every hero at once, then 0x1E2BE
                self.w.scripted = 0;
                self.script_end();
            }
            0x0E => self.script_walk(i, Walk::Start), // 0x1E062
            0x10 => self.script_walk_point(i), // 0x1E2FC
            0x12 => self.script_hold_right(i), // 0x1E4BA
            0x14 => self.script_walk(i, Walk::Spots), // 0x1E4C8
            0x16 => self.script_jump_kick(i), // 0x1E54A
            0x18 => self.script_hold_right_jump(i), // 0x1E5C8
            0x1A => {
                self.players[i].b.facing = 1; // 0x1E5D6: face right and fall into the area (air state 0x1C, 0x1AF82); no pad clear
                self.player_scene_fall_entry(i);
            }
            0x1C => {} // 0x1E5E2: nothing, the pad passes through
            0x1E => self.script_walk(i, Walk::Lineup), // 0x1E11E
            0x20 => self.script_wait_flag(i, self.spawn.g.b4fc != 0), // 0x1DEE2: the boss intro G.$4FC
            0x22 => {
                self.prop_caption_spawn(self.flow.script.b4fd as u16); // 0x1DED4: the caption banner of the record's argument (0x86D2E)
                self.script_pad_clear();
            }
            0x24 => {
                self.spawn_script_prop(0x24); // 0x1DECC: pool B class 21, the tile change of the hall (0x7BDAE)
                self.script_pad_clear();
            }
            0x26 => self.script_end(), // 0x1E2BE: this hero only (his bit went out of G.$4DD with the fetch)
            0x28 => {
                self.flow.script.b4d8 = 1; // 0x1DEA0
                self.script_pad_clear();
            }
            0x2C => {
                self.cam.s2.go = 1; // 0x1DE62: G.$7A9
                self.script_pad_clear();
            }
            0x2A => self.script_wait_items(i, true, 0x5A), // 0x1DE6C
            0x2E => self.script_wait_flag(i, self.flow.b4e2 != 0), // 0x1DE44
            0x30 => self.script_wait_ground_present(i), // 0x1DE02
            0x32 => self.script_jump_to_point(i), // 0x1DD4C
            0x34 => self.script_wait_ground_scripted(i), // 0x1DDC8
            0x3A => {
                // 0x1DD3A: two small hops (state 0x2A, 0x19EA6), once
                if self.players[i].sc.step == 0 {
                    self.players[i].sc.step += 2;
                    self.player_hops_entry(i);
                }
            }
            0x3E => {
                // 0x1E2AA: the leader's pose with the effect at the chest (state 0x2C, 0x19F40), then 0x1DEA6
                if self.players[i].bc0 != 0 {
                    self.player_chest_pose_entry(i);
                }
                self.script_pad_clear();
            }
            0x36 => {
                self.cam.s2.go = 1; // 0x1E4B4: go, then hold right
                self.script_hold_right(i);
            }
            0x38 => self.script_wait_items(i, false, 0xA0), // 0x1DE86
            0x3C => self.script_walk_right_to_b80(i), // 0x1E474
            0x40 => {
                self.script_pad_clear(); // 0x1DD30: the door smash, pool B class 11 (0x789C8)
                self.spawn_script_prop(0x40);
            }
            0x44 => self.flow.b4e2 = 0, // 0x1DD2A (no pad clear)
            0x48 => {
                // 0x1DCFE: the map card is running (only in a game)
                if self.flow.in_game != 0 {
                    self.flow.b4e2 = 1;
                }
                self.script_pad_clear();
            }
            0x46 => {
                self.spawn_script_prop(0x46); // 0x1DD0E: the forced GO blink, pool B class 20 variant 2 (no pad clear)
            }
            0x42 => self.script_clear_sequence(i), // 0x1E650
            0x4A => self.script_barrier(i), // 0x1DC60
            0x4C => self.script_wait_flag(i, self.spawn.g.car_mode != 0), // 0x1DC42: the car G.$518
            0x4E => {
                self.sound(self.flow.script.b4fd as u16); // 0x1DC28: the sound of the record's argument (0xA63C)
                self.script_pad_clear();
            }
            0x50 => {
                self.hook_a5ea(); // 0x1DC38: the music fade out (0xA5EA)
                self.script_pad_clear();
            }
            _ => {}
        }
    }

    // ------------------------------------------------------------------------------------------------ shared pieces

    /// 0x1DEA6: clears `+$A0`, `+$A8` and `+$BC` of all three heroes (the pad the actions have written, held and pressed, and the
    /// double tap timer).
    pub fn script_pad_clear(&mut self) {
        for p in self.players.iter_mut() {
            p.held = 0;
            p.pressed = 0;
            p.dtap = 0;
        }
    }

    /// 0x1E5E4: `+$A8` = the bits of `+$A0` that were not in `+$A1` (the new presses of the pad the action has made).
    fn script_pressed_edges(&mut self, i: usize) {
        let p = &mut self.players[i];
        p.pressed = p.held & (p.held ^ p.prev);
    }

    /// The wait actions 0x20, 0x2E and 0x4C: timer 0xFFFF while the flag is set (run again next frame), else 1; then 0x1DEA6.
    fn script_wait_flag(&mut self, i: usize, waiting: bool) {
        self.players[i].sc.timer = if waiting { 0xFFFF } else { 1 };
        self.script_pad_clear();
    }

    /// The end of a script (0x1E2BE): G.$4E3, G.$4D8, `+$D0` of all heroes and the pad bytes `+$A0`, `+$A8` of all heroes are cleared. The
    /// action 0x0A has cleared G.$4DD first; the action 0x26 has not (the heroes whose bit is set go on, and start their script again
    /// in this frame: their `+$D0` is 0 now).
    fn script_end(&mut self) {
        self.flow.b4e3 = 0;
        self.flow.script.b4d8 = 0;
        for p in self.players.iter_mut() {
            p.bd0 = 0; // 0x3344, 0x34C4, 0x3644
            p.held = 0; // 0x3314, 0x3494, 0x3614
            p.pressed = 0; // 0x331C, 0x349C, 0x361C
        }
    }

    /// 0x1E28C action 0x08: the leader presses attack (held and pressed), the others release everything. No timer, no 0x1DEA6.
    fn script_leader_attack(&mut self, i: usize) {
        let p = &mut self.players[i];
        let pad = if p.bc0 != 0 { ATK } else { 0 };
        p.pressed = pad;
        p.held = pad;
    }

    /// 0x1E4BA action 0x12 (and the second half of 0x36): hold right, held and pressed, every frame.
    fn script_hold_right(&mut self, i: usize) {
        let p = &mut self.players[i];
        p.held = RIGHT;
        p.pressed = RIGHT;
    }

    /// 0x1E5C8 action 0x18: hold right and jump (pad 0x21), held and pressed.
    fn script_hold_right_jump(&mut self, i: usize) {
        let p = &mut self.players[i];
        p.held = JMP | RIGHT;
        p.pressed = JMP | RIGHT;
    }

    // ------------------------------------------------------------------------------------------------ waits

    /// 0x1DE02 action 0x30: wait until every present hero is on the ground (`+$51` clear).
    fn script_wait_ground_present(&mut self, i: usize) {
        let air = self.players.iter().filter(|p| p.b.live != 0).fold(0u8, |a, p| a | p.b.airborne);
        self.players[i].sc.timer = if air != 0 { 0xFFFF } else { 1 };
        self.script_pad_clear();
    }

    /// 0x1DDC8 action 0x34: wait until every scripted hero (the bits of G.$4DD) is on the ground.
    fn script_wait_ground_scripted(&mut self, i: usize) {
        let mask = self.w.scripted;
        let air = (0..3).filter(|k| mask & (1 << k) != 0).fold(0u8, |a, k| a | self.players[k].b.airborne);
        self.players[i].sc.timer = if air != 0 { 0xFFFF } else { 1 };
        self.script_pad_clear();
    }

    /// 0x32762 (`with_class0`) and 0x327CA: a floor item is around when a pool E object of the class 0 (only 0x32762), 2, 4, 5 or 9
    /// (`+$20` = 0, 8, 0x10, 0x14, 0x24: weapons, food, containers, treasure) is in the mode 0 to 2 and its sub is 0 or 0xC (lying around, not
    /// being picked up or broken).
    pub(crate) fn script_floor_items(&self, with_class0: bool) -> bool {
        self.items.slots.iter().any(|it| {
            let b = &it.b;
            b.live != 0
                && (matches!(b.k20, 8 | 0x10 | 0x14 | 0x24) || (with_class0 && b.k20 == 0))
                && b.mode <= 2
                && (b.sub == 0 || b.sub == 0xC)
        })
    }

    /// 0x1DE6C action 0x2A and 0x1DE86 action 0x38: wait for the floor items to be taken or broken, at most the record's time. On the first run,
    /// while an item is around the timer stays what the record says (120, 180 or 240: the longest wait); when none is left (the test is
    /// made in the first frame only, `+$D3` = 2 after it) the timer is set to `after` (90 or 160) frames. The pad passes through.
    fn script_wait_items(&mut self, i: usize, with_class0: bool, after: u16) {
        if self.players[i].sc.step != 0 {
            return;
        }
        if self.script_floor_items(with_class0) {
            return;
        }
        let s = &mut self.players[i].sc;
        s.timer = after;
        s.step += 2;
    }

    /// 0x1E650 action 0x42: the clear sequence of Episode 8 (the end of script 24: the last boss is dead, the heroes have walked to their
    /// end spots). The first run of the first hero (`+$D3` = 0 and G.-$70AA = 0) starts it for everybody: G.-$70AA = 1, G.$516 = 1 (the
    /// player update skips the hero state machine, `player.rs`), the sound 0x23, and one pool A class 97 stand in for each of the four
    /// heroes: the present ones (0x69E32, copied from the player) and, for the characters nobody plays (the bits of d5 that are not set, from 3
    /// down to 0), the absent ones (0x69E66, at X 0x10E0). Every run of every hero then sets the timer: 1 (the record ends) when the boss's `+$A6`
    /// (the object G.$4E0) is not 0, 0xFFFF (run again next frame) while it is 0. No pad clear (the action ends with `rts`). D4c 5.4 has the
    /// wait the wrong way round (D5 errata).
    ///
    /// The stand ins are pool A class 97 objects (M3: the ending); until then `script_stand_in_present` / `_absent` only mark G.-$70A9.
    fn script_clear_sequence(&mut self, i: usize) {
        if self.players[i].sc.step == 0 && self.flow.script.b70aa == 0 {
            self.players[i].sc.step += 2; // 0x1E660
            self.flow.script.b70aa = 1;
            self.flow.script.b516 = 1;
            self.sound(0x23); // moveq #$23,d0 / jsr 0xA63C
            let mut present = 0u8; // d5
            for k in 0..3 {
                if self.players[k].b.live != 0 {
                    let id = self.players[k].b.id & 7; // `+$03`: the character
                    present |= 1 << id;
                    self.flow.script.b70a9 |= 1 << id;
                    self.script_stand_in_present(k, id);
                }
            }
            for id in (0..4u8).rev() {
                if present & (1 << id) == 0 {
                    present |= 1 << id;
                    self.script_stand_in_absent(id);
                }
            }
        }
        // 0x1E6E2: the boss `+$A6`
        let boss = self.spawn.g.killer; // movea.w $4E0(a5),a0
        let a6 = match super::slot_of(boss) {
            Some(super::Slot::Enemy(e)) => self.enemies.slots[e].m.b(0xA6),
            _ => 0,
        };
        self.players[i].sc.timer = if a6 != 0 { 1 } else { 0xFFFF };
    }

    /// 0x69E32 (called by 0x1E650 for a present hero, d0 = his character): a pool A class 97 object (`+$20` = 0x184, variant `2 * id`) at the
    /// hero's position and facing. Not ported (M3-27, the staff roll and the clear scene): the hook is here so the action is complete.
    fn script_stand_in_present(&mut self, hero: usize, id: u8) {
        self.enemy_stand_in_present(hero, id);
    }

    /// 0x69E66 (called by 0x1E650 for a character nobody plays, d0 = its id): a pool A class 97 object (variant `2 * id`) at X 0x10E0, height 0,
    /// facing 1, the depth from the table 0x69E9A. Not ported (M3-27).
    fn script_stand_in_absent(&mut self, id: u8) {
        self.enemy_stand_in_absent(id);
    }

    /// 0x1DC60 action 0x4A: the barrier. Waits until every scripted hero has reached this action, has HP and is not in the invulnerable
    /// state (`+$00` = 2, after a hit or a respawn). A scripted hero whose object is free (`+$00` = 0) is taken out of G.$4DD, but the frame still
    /// waits (his action is not this one). The bits are tested on the copy of G.$4DD from the start of the routine.
    fn script_barrier(&mut self, i: usize) {
        let mask = self.w.scripted;
        let mut wait = false;
        for k in 0..3 {
            if mask & (1 << k) == 0 {
                continue;
            }
            if self.players[k].b.live == 0 {
                self.w.scripted &= !(1 << k);
            }
            let p = &self.players[k];
            if p.sc.act != 0x4A || p.b.hp == 0 || p.b.live == 2 {
                wait = true;
                break;
            }
        }
        self.players[i].sc.timer = if wait { 0xFFFF } else { 1 };
        self.script_pad_clear();
    }

    // ------------------------------------------------------------------------------------------------ walks

    /// The actions 0x02, 0x0E, 0x1E and 0x14: on the first run the targets are chosen, then the walk body runs every frame.
    fn script_walk(&mut self, i: usize, which: Walk) {
        if self.players[i].sc.step == 0 {
            match which {
                Walk::End => self.script_targets_end(i), // 0x1DFCA
                Walk::Start => self.script_targets_start(i), // 0x1E07E
                Walk::Lineup => self.script_targets_lineup(), // 0x1E13A
                Walk::Spots => self.script_targets_spots(i), // 0x1E4E4
            }
            self.players[i].sc.count = 0;
            self.flow.script.b4df = 0;
            self.players[i].sc.step += 2;
        }
        self.script_walk_body(i);
    }

    /// 0x1DFCA: the target of the end walk (table 0x1EE6A: three spots by the stage, 16 bytes a row, the cabinet row +4 on a two player
    /// machine).
    fn script_targets_end(&mut self, i: usize) {
        let off = 16 * self.w.stage as u32 + if self.flow.b4cf != 0 { 4 } else { 0 }; // 0x1DFD8
        self.script_take_spot(i, tab::END_TARGETS, off);
    }

    /// 0x1E07E: the target of the start walk (the pointer table 0x1EEEA by the stage, 16 bytes an area). The test of G.$4CF in the original
    /// changes nothing (its flags are overwritten by the next instruction).
    fn script_targets_start(&mut self, i: usize) {
        let t = &self.assets.script.tables;
        let row = t.u32(tab::START_TARGETS, 4 * self.w.stage as u32) + 16 * self.cam.s2.area as u32;
        self.script_take_spot(i, row, 0);
    }

    /// The common part of 0x1DFCA and 0x1E07E: the leader takes the spot 0, any other hero the free spot (1 or 2) that is nearest to him
    /// (a spot another hero has taken has the distance 0xFFFF). The slot goes to `+$DA` and to G.$4EC.., the spot to `+$DC` and `+$DE`.
    fn script_take_spot(&mut self, i: usize, table: u32, off: u32) {
        let t = &self.assets.script.tables;
        let spot = |k: u32| (t.u16(table, off + 4 * k), t.u16(table, off + 4 * k + 2));
        let p = &self.players[i];
        let slot = if p.bc0 != 0 {
            0
        } else {
            let (x, z) = (p.b.xi() as u16, p.b.zi() as u16);
            let near = |k: u32, taken: u8| {
                let (sx, sz) = spot(k);
                if taken != 0 { 0xFFFF } else { distance(x, z, sx, sz) }
            };
            let d = [0xFFFF, near(1, self.flow.script.taken[1]), near(2, self.flow.script.taken[2])];
            let mut best = 0usize;
            for k in 1..3 {
                if d[k] < d[best] {
                    best = k; // cmp.w d2,d3 / bcc: only a smaller distance takes over
                }
            }
            best
        };
        let (tx, tz) = spot(slot as u32);
        self.flow.script.taken[slot] = 1;
        let s = &mut self.players[i].sc;
        s.rank = slot as u8;
        s.tx = tx;
        s.tz = tz;
    }

    /// 0x1E170: the present heroes, packed to the front in the order of the players and then sorted by X, the right most first (signed words;
    /// two steps of exchange as the original does: a1 against a2, then a2 against a3 and again a1 against a2).
    fn script_sorted_heroes(&self) -> Vec<usize> {
        let mut v: Vec<usize> = (0..3).filter(|&k| self.players[k].b.live != 0).collect();
        let x = |k: usize| self.players[k].b.xi();
        if v.len() >= 2 && x(v[0]) < x(v[1]) {
            v.swap(0, 1);
        }
        if v.len() == 3 && x(v[1]) < x(v[2]) {
            v.swap(1, 2);
            if x(v[0]) < x(v[1]) {
                v.swap(0, 1);
            }
        }
        v
    }

    /// 0x1E13A: the lineup spots of the stage (table 0x1F10A) go to the present heroes, the right most hero to the front spot.
    fn script_targets_lineup(&mut self) {
        let heroes = self.script_sorted_heroes();
        let off = 16 * self.w.stage as u32;
        let t = &self.assets.script.tables;
        for (k, &h) in heroes.iter().enumerate() {
            let s = &mut self.players[h].sc;
            s.tx = t.u16(tab::LINEUP, off + 4 * k as u32);
            s.tz = t.u16(tab::LINEUP, off + 4 * k as u32 + 2);
        }
    }

    /// 0x1E4E4: one of three fixed spots (table 0x1E532, a second bank for stage 7) by the order of the scripted heroes; one player: the third spot.
    fn script_targets_spots(&mut self, i: usize) {
        let bank = if self.w.stage == 7 { 12 } else { 0 };
        let k = if self.w.players_minus_one != 0 {
            let who = self.players[i].b.who as usize;
            (0..who).filter(|&j| self.w.scripted & (1 << j) != 0).count() as u32
        } else {
            2
        };
        let t = &self.assets.script.tables;
        let s = &mut self.players[i].sc;
        s.tx = t.u16(tab::SPOTS, bank + 4 * k);
        s.tz = t.u16(tab::SPOTS, bank + 4 * k + 2);
    }

    /// 0x1DF10: the walk. The direction of the last frame stays (`+$A0 = +$A1`); every fifth frame (`+$DB` counts 4 to 0) it is recomputed from
    /// the target: right unless the target is left of the hero (unsigned words), none inside a dead zone of 4 pixels; away from the viewer
    /// unless the target is nearer (Z); and the arrival test is made (a box of 0xC x 0x18, 0x10 x 0x20 in stage 2). A hero who arrived presses
    /// nothing and turns right; when all scripted heroes have arrived the timers go to 1.
    fn script_walk_body(&mut self, i: usize) {
        let stage = self.w.stage;
        let p = &mut self.players[i];
        p.held = p.prev;
        let n = p.sc.count;
        p.sc.count = n.wrapping_sub(1);
        if n != 0 {
            return self.script_pressed_edges(i); // bcc 0x1DF96
        }
        p.sc.count = 4;
        let (x, z) = (p.b.xi() as u16, p.b.zi() as u16);
        let (tx, tz) = (p.sc.tx, p.sc.tz);
        let mut d0 = if tx < x { LEFT } else { RIGHT };
        if tx.wrapping_sub(x).wrapping_add(4) < 8 {
            d0 = 0;
        }
        let mut d1 = if tz < z { DOWN } else { UP };
        if tz.wrapping_sub(z).wrapping_add(4) < 8 {
            d1 = 0;
        }
        p.held = d0 | d1;
        let (half, width) = if stage == 2 { (0x10u16, 0x20u16) } else { (0xC, 0x18) };
        if x.wrapping_sub(tx).wrapping_add(half) < width && z.wrapping_sub(tz).wrapping_add(half) < width {
            self.script_arrived(i);
            self.players[i].b.facing = 1;
        }
        self.script_pressed_edges(i);
    }

    /// 0x1DF9A: the hero is at his spot: his bit in G.$4DF; when it equals G.$4DD the timers of all three heroes go to 1 (the record ends in
    /// the next frame). The hero presses nothing.
    fn script_arrived(&mut self, i: usize) {
        let who = self.players[i].b.who & 7;
        self.flow.script.b4df |= 1 << who;
        if self.w.scripted == self.flow.script.b4df {
            for p in self.players.iter_mut() {
                p.sc.timer = 1; // 0x334C, 0x34CC, 0x364C
            }
        }
        let p = &mut self.players[i];
        p.held = 0;
        p.pressed = 0;
    }

    /// 0x1E2FC action 0x10: all heroes walk to one point (table 0x1E3F4 by stage and area). Unlike the others this walk recomputes the direction
    /// every frame, tests a box of 0xC x 0x18 (also in stage 2) and ends the record of this hero alone (timer 1) when he is there.
    fn script_walk_point(&mut self, i: usize) {
        if self.players[i].sc.step == 0 {
            // 0x1E3BE: the point goes to all three heroes
            let t = &self.assets.script.tables;
            let off = 16 * self.w.stage as u32 + 4 * self.cam.s2.area as u32;
            let (px, pz) = (t.u16(tab::POINTS, off), t.u16(tab::POINTS, off + 2));
            for p in self.players.iter_mut() {
                p.sc.tx = px;
                p.sc.tz = pz;
            }
            self.players[i].sc.count = 0;
            self.flow.script.b4df = 0;
            self.players[i].sc.step += 2;
        }
        let p = &mut self.players[i];
        p.held = p.prev;
        let (x, z) = (p.b.xi() as u16, p.b.zi() as u16);
        let (tx, tz) = (p.sc.tx, p.sc.tz);
        let mut d0 = if tx < x { LEFT } else { RIGHT };
        if tx.wrapping_sub(x).wrapping_add(4) < 8 {
            d0 = 0;
        }
        let mut d1 = if tz < z { DOWN } else { UP };
        if tz.wrapping_sub(z).wrapping_add(4) < 8 {
            d1 = 0;
        }
        p.held = d0 | d1;
        if x.wrapping_sub(tx).wrapping_add(0xC) < 0x18 && z.wrapping_sub(tz).wrapping_add(0xC) < 0x18 {
            p.sc.timer = 1; // 0x1E3B6
            p.b.facing = 1;
        }
        self.script_pressed_edges(i);
    }

    /// 0x1E474 action 0x3C: walk right until X >= 0xB80 (signed), then stand and count as arrived (0x1DF9A).
    fn script_walk_right_to_b80(&mut self, i: usize) {
        if self.players[i].sc.step == 0 {
            let p = &mut self.players[i];
            p.sc.step += 2;
            p.sc.tx = 0xB80;
            p.sc.tz = p.b.zi() as u16;
            p.sc.count = 0;
            self.flow.script.b4df = 0;
        }
        let p = &mut self.players[i];
        if p.b.xi() < p.sc.tx as i16 {
            p.held = RIGHT;
            p.pressed = RIGHT;
            return;
        }
        self.script_pad_clear();
        self.script_arrived(i);
    }
}

/// 0x43390 (the sign fix of `d2.w`) and 0x4339A: the quotient and the remainder of `|d2.w| / d3`, bit by bit as the original divides
/// (no divisor: both 0).
fn divide_abs(d2: u16, d3: u32) -> (u32, u32) {
    let d2 = if (d2 as i16) < 0 { d2.wrapping_neg() } else { d2 } as u32; // tst.w d2 / bpl / neg.w d2
    if d3 == 0 {
        return (0, 0);
    }
    (d2 / d3, d2 % d3)
}

/// The jump of the action 0x32 and the area end jump of Episodes 7 and 8: the time in frames (0x1BBF4: moveq #$2F,d3).
const SCRIPTED_JUMP_FRAMES: u32 = 0x2F;

impl Game {
    // ------------------------------------------------------------------------------------------------ the actions that start player states

    /// 0x1E200 action 0x04: the pose. Waits (timer 0xFFFF) until every scripted hero has HP and is on the ground; then G.$503 = 1 (the pose
    /// lock), the timer 1, sound 0x18 and the pose state of this hero (0x19996). The others follow in the same frame, in the order of the heroes.
    /// The pose state is mode 2 sub 0, so the script goes on while he poses (the pose holds as long as G.$503 is set, `player_pose_hold`).
    fn script_pose(&mut self, i: usize) {
        let mask = self.w.scripted;
        let mut air = 0u8;
        let mut dead = false;
        for k in 0..3 {
            if mask & (1 << k) == 0 {
                continue;
            }
            air |= self.players[k].b.airborne;
            if self.players[k].b.hp == 0 {
                dead = true; // beq 0x1E258
                break;
            }
        }
        if dead || air != 0 {
            self.players[i].sc.timer = 0xFFFF;
            self.script_pad_clear();
            return;
        }
        self.flow.script.b503 = 1;
        self.players[i].sc.timer = 1;
        self.script_pad_clear();
        self.sound(0x18); // moveq #$18,d0 / jsr 0xA63C
        self.player_pose_entry(i); // jmp 0x19996
    }

    /// 0x1E262 action 0x06: the scene is done (the opening has finished or the heroes have walked off after a boss): once G.$4E3 = 1 (the
    /// game task starts the tally at it, the intro task slides the HUD in) and the timer 0xFFFF; then it waits while the pose lock G.$503 is set
    /// (the clear task clears it). The pad is cleared every frame.
    fn script_scene_done(&mut self, i: usize) {
        if self.players[i].sc.step == 0 {
            self.players[i].sc.step += 2;
            self.flow.b4e3 = 1;
            self.players[i].sc.timer = 0xFFFF;
        } else if self.flow.script.b503 == 0 {
            self.players[i].sc.timer = 1;
        }
        self.script_pad_clear();
    }

    /// 0x1E2EE action 0x0C: the special attack: sound 2 and the special of the hero (0x1968E, the state 0x0E). No pad clear.
    fn script_special(&mut self, i: usize) {
        self.sound(2);
        self.player_special_entry(i);
    }

    /// 0x1E54A action 0x16: the jump kick. The first run: `+$DB` = the walk rank (`+$DA`), the pad cleared, facing right. Then every hero counts
    /// his rank down and jumps right (pad 0x21, held and pressed) when it is below 0, so the heroes jump one after the other; `+$DA` = 0xFF
    /// from then on. The leader: Jack (id 0) waits 10 frames after the jump and presses attack (0x10, a frame with `+$DB` = 0); the
    /// other characters do not jump at all: they start the dash attack state of the leader (0x19720: the lunge).
    fn script_jump_kick(&mut self, i: usize) {
        if self.players[i].sc.step == 0 {
            let p = &mut self.players[i];
            p.sc.count = p.sc.rank;
            p.held = 0;
            p.pressed = 0;
            p.b.facing = 1;
            p.sc.step += 2;
            return;
        }
        let p = &mut self.players[i];
        if (p.sc.rank as i8) >= 0 {
            p.sc.count = p.sc.count.wrapping_sub(1);
            if (p.sc.count as i8) < 0 {
                p.sc.rank = 0xFF;
                if p.bc0 != 0 {
                    if p.b.id != 0 {
                        return self.player_dash_attack_entry(i); // jmp 0x19720
                    }
                    p.sc.count = 0xA;
                }
                p.held = JMP | RIGHT;
                p.pressed = JMP | RIGHT;
            }
        }
        // 0x1E5A8: the Jack leader presses attack when the count reaches 0
        let p = &mut self.players[i];
        if p.bc0 != 0 && p.b.id == 0 {
            p.sc.count = p.sc.count.wrapping_sub(1);
            if p.sc.count == 0 {
                p.held = ATK;
                p.pressed = ATK;
            }
        }
    }

    /// 0x1DD4C action 0x32: the scripted jump. The first run: the target X = X + the offset of the stage and area (table 0x1DD88, signed
    /// words; 0 for most areas, 0x100 in Episode 7 area 0, 0x40 and 0x120 in Episode 8 areas 0 and 1), the target Z = Z (`+$88` and `+$8A`), then the
    /// jump state 0x34 (0x1BBCA). Afterwards the pad is cleared.
    fn script_jump_to_point(&mut self, i: usize) {
        if self.players[i].sc.step != 0 {
            self.script_pad_clear();
            return;
        }
        let off = self.assets.script.tables.u16(0x1DD88, 8 * self.w.stage as u32 + 2 * self.cam.s2.area as u32);
        let p = &mut self.players[i];
        p.w88 = (p.b.xi() as u16).wrapping_add(off);
        p.w8a = p.b.zi() as u16;
        p.sc.step += 2;
        self.player_scripted_jump_entry(i);
    }

    // ------------------------------------------------------------------------------------------------ player states of the scenes

    /// 0x19EA6 state 0x2A entry (action 0x3A): two small hops, walking with the pose of a held item. Falls into the handler.
    pub(super) fn player_hops_entry(&mut self, i: usize) {
        let p = &mut self.players[i];
        p.b.set_state(0x0200_002A);
        p.b90 = 0;
        self.player_hops(i);
    }

    /// 0x19EB2 state 0x2A (the steps `+$90` 0, 2, 4 at 0x19EC6, 0x19EEC, 0x19F14): step 0 starts to move right (0xC0) and to hop in depth (the
    /// acceleration -12 for 13 frames, then 9 frames of rest, and again for ever: only another state ends it).
    pub(super) fn player_hops(&mut self, i: usize) {
        match self.players[i].b90 {
            0 => {
                let p = &mut self.players[i];
                p.b.vx = 0xC0;
                p.b.ax = 0;
                p.b.vz = 0;
                p.b.az = -12;
                p.set_b80(0xC);
                p.b90 += 2;
                self.player_walk_anim(i); // jsr 0x5CA6
                self.player_hops(i); // falls into 0x19EEC
            }
            2 | 4 => {
                self.player_pmove(i); // jsr 0x189C
                self.player_pstep(i); // jsr 0x1426
                let p = &mut self.players[i];
                let old = p.b80();
                p.set_b80(old.wrapping_sub(1));
                if old != 0 {
                    return; // subq.b / bcc: no borrow
                }
                if p.b90 == 2 {
                    p.set_b80(8);
                    p.b.vz = 0;
                    p.b.az = 0;
                    p.b90 += 2;
                } else {
                    p.set_b80(0xC);
                    p.b.vz = 0;
                    p.b.az = -12;
                    p.b90 = 2;
                }
            }
            _ => {}
        }
    }

    /// 0x19996 state 0x18 entry: the pose (action 0x04, and the extra life pose). The animation 0xE for a hero with the flag `+$10E` (a life was won) or
    /// the leader, 0x11 for the others. Mustapha (id 1) and Hannah (id 2) in these two cases use the air states 0x418 (0x1ADA6) and 0x41A
    /// (0x1AE62) instead. The leader (and not an extra life) spawns the bubbles 0x6192 before the voice; an extra life
    /// skips that spawn. The anti tamper test 0x5124E (it never fails with the real ROM) is not ported. Falls into the handler.
    pub(super) fn player_pose_entry(&mut self, i: usize) {
        let p = &mut self.players[i];
        p.b.set_state(0x0200_0018);
        p.b.draw_flags = 0;
        let leader = p.bc0 != 0;
        let extra = p.b10e != 0;
        let special = extra || leader;
        // 0x199A4: +$10E skips the spawn and still voices; a leader with no extra life does jsr 0x6192 then jsr 0x9814.
        if leader && !extra {
            self.fx_spawn_bubbles(Game::player_handle(i));
        }
        if special {
            self.player_oneup_voice(i);
        }
        let anim = if special { 0xE } else { 0x11 };
        self.player_panim(i, 0, anim);
        let p = &mut self.players[i];
        p.b90 = 0;
        if special {
            match p.b.id {
                1 => return self.player_pose_air_entry(i, false), // bra 0x1ADA6
                2 => return self.player_pose_air_entry(i, true),  // bra 0x1AE62
                _ => {}
            }
        }
        self.player_pose(i);
    }

    /// 0x19A0A state 0x18: step 0 plays the animation until it ends or the frame flag `+$2C` is set (then `+$90` = 2, and with a won life the timer
    /// `+$11D` = 0x14 and the "1UP!" effect 0x18C1E); step 2 holds the pose while the pose lock G.$503 is set (the clear task clears it) or,
    /// with a won life, for `+$11D` frames with the palette flash 0x1BD78, then goes idle (0x19122).
    pub(super) fn player_pose(&mut self, i: usize) {
        if self.players[i].b90 == 0 {
            let done = self.player_pstep(i);
            let p = &mut self.players[i];
            if !done && p.b.param_a == 0 {
                return;
            }
            p.b90 += 2;
            if p.b10e != 0 {
                p.b11d = 0x14;
                self.fx_spawn_player_marker(i, 2); // 0x19A38 jsr 0x18C1E, the "1UP!" effect
            }
            return;
        }
        self.player_pose_hold(i);
    }

    /// The end of the pose states 0x18 / 0x418 / 0x41A (0x19A40, 0x1AE26, 0x1AF46): without a won life the pose lasts while G.$503 is set; with
    /// one the palette flashes (+$23 = 0x8A while `+$11D & 3` = 3) and `+$11D` counts down from 0x14; at the end the life flag `+$10E` and the timer
    /// `+$11C` are cleared, the hero is invulnerable for 0xB4 frames (`+$EC`, `+$00` = 2). Both end in `+$23` = 0 and the idle entry.
    fn player_pose_hold(&mut self, i: usize) {
        if self.players[i].b10e == 0 {
            if self.flow.script.b503 != 0 {
                return;
            }
        } else {
            // 0x1BD78: the palette override flashes while the two low bits of `+$11D` are 3
            let p = &mut self.players[i];
            p.b.palette_override = if p.b11d & 3 == 3 { 0x8A } else { 0 };
            let old = p.b11d;
            p.b11d = old.wrapping_sub(1);
            if old != 0 {
                return;
            }
            p.b10e = 0;
            p.b11c = 0;
            p.respawn_t = 0xB4;
            p.b.live = 2;
        }
        self.players[i].b.palette_override = 0; // clr.b $23(a6)
        self.player_idle_entry(i);
    }

    /// 0x1ADA6 (`hannah` false: Mustapha) and 0x1AE62 (`hannah` true) the entries of the air poses 0x418 and 0x41A: still in the air, no sideways
    /// speed, up at 0x300 and the acceleration -0x45; they fall into the handlers (0x1ADCC, 0x1AE88).
    fn player_pose_air_entry(&mut self, i: usize, hannah: bool) {
        let p = &mut self.players[i];
        p.b.set_state(if hannah { 0x0200_041A } else { 0x0200_0418 });
        p.b.vz = 0;
        p.b.az = 0;
        p.b.vx = 0;
        p.b.ax = 0;
        p.b.vy = 0x300;
        p.b.ay = -0x45;
        if hannah {
            self.player_pose_air_hannah(i);
        } else {
            self.player_pose_air_mustapha(i);
        }
    }

    /// 0x1ADCC state 0x418 (Mustapha's pose): step 0 jumps; while falling it tests the splash and the floor, plays the animation and moves until the frame
    /// flag `+$2C` or the end of the animation; step 2 holds like 0x18.
    pub(super) fn player_pose_air_mustapha(&mut self, i: usize) {
        if self.players[i].b90 != 0 {
            return self.player_pose_hold(i); // 0x1AE26
        }
        if self.players[i].b.vy < 0 {
            self.player_splash_check(i); // jsr 0x606E
            if self.players[i].b.airborne == 0 {
                self.players[i].b.vy = 0;
                self.players[i].b.ay = 0;
            }
        }
        let done = self.player_pstep(i);
        if !done && self.players[i].b.param_a == 0 {
            return self.player_pmove(i); // jmp 0x189C
        }
        let p = &mut self.players[i];
        p.b90 += 2;
        if p.b10e != 0 {
            p.b11d = 0x14;
            self.fx_spawn_player_marker(i, 2); // 0x1AE1E jsr 0x18C1E (the "1UP!" effect)
        }
    }

    /// 0x1AE88 state 0x41A (Hannah's pose): step 0 plays the animation until the flag `+$2C` (then jmp 0x965A2, the smoke ring variant 0);
    /// step 2 falls and bounces (`vy` = -`vy` when it lands), step 4 plays to the flag (jmp 0x965A2 variant 2), step 6 falls again and ends
    /// with the won life effect, step 8 plays and holds like 0x18. The low-address copies of the same jumps are 0x378E and 0x37D0.
    pub(super) fn player_pose_air_hannah(&mut self, i: usize) {
        match self.players[i].b90 {
            0 => {
                self.player_pstep(i);
                if self.players[i].b.param_a != 0 {
                    self.players[i].b90 += 2;
                    self.fx_spawn_smoke_ring(Game::player_handle(i), 0); // jmp 0x965A2, d0 = 0
                }
            }
            2 => {
                if self.players[i].b.vy < 0 {
                    self.player_splash_check(i);
                    if self.players[i].b.airborne == 0 {
                        let p = &mut self.players[i];
                        p.b.vy = p.b.vy.wrapping_neg();
                        p.b90 += 2;
                        return;
                    }
                }
                self.player_pmove(i);
            }
            4 => {
                self.player_pstep(i);
                if self.players[i].b.param_a != 0 {
                    self.players[i].b90 += 2;
                    self.fx_spawn_smoke_ring(Game::player_handle(i), 2); // jmp 0x965A2, d0 = 2
                }
            }
            6 => {
                if self.players[i].b.vy < 0 {
                    self.player_splash_check(i);
                    if self.players[i].b.airborne == 0 {
                        self.players[i].b90 += 2;
                        self.player_pstep(i);
                        if self.players[i].b10e != 0 {
                            self.players[i].b11d = 0x14;
                            return self.fx_spawn_player_marker(i, 2); // 0x1AF2C jmp 0x18C1E
                        }
                    }
                }
                self.player_pmove(i); // 0x1AF32
            }
            8 => {
                if self.players[i].b.param_a == 0 {
                    self.player_pstep(i);
                }
                self.player_pose_hold(i); // 0x1AF46
            }
            _ => {}
        }
    }

    /// 0x1AF82 state 0x1C entry (action 0x1A): the fall into Episode 1 area 2. The heroes come down from above the area with the speed 0x500 to the right
    /// (decelerating by 13), up at 0x300 and the acceleration -0x20; the animation 0x14 (0x15 for the leader), no shadow (`+$25` = 0x10) until
    /// they are lower than 0x100. Falls into the handler.
    pub(super) fn player_scene_fall_entry(&mut self, i: usize) {
        let p = &mut self.players[i];
        p.b.set_state(0x0200_041C);
        self.player_count_players(); // jsr 0x655E
        let p = &mut self.players[i];
        p.b.vz = 0;
        p.b.az = 0;
        p.b.vx = 0x500;
        p.b.ax = -13;
        p.b.vy = 0x300;
        p.b.ay = -0x20;
        p.b.airborne = 1;
        p.b.draw_flags = 0x10;
        let anim = if p.bc0 != 0 { 0x15 } else { 0x14 };
        self.player_panim(i, 0, anim);
        self.player_scene_fall(i);
    }

    /// 0x1AFD0 state 0x1C: the sideways speed stops at 0, the fall speed is at most -0x800; below the height 0x100 the shadow comes back; in the
    /// air the hero moves and the animation steps; on the floor the screen shakes (G.$755C = 1), `live` = 1 and the landing state follows (0x1A81A).
    pub(super) fn player_scene_fall(&mut self, i: usize) {
        let p = &mut self.players[i];
        if p.b.vx < 0 {
            p.b.vx = 0;
            p.b.ax = 0;
        }
        if p.b.vy < -0x800 {
            p.b.vy = -0x800;
            p.b.ay = 0;
        }
        if (p.b.yi() as u16) < 0x100 {
            p.b.draw_flags = 0;
        }
        if p.b.airborne != 0 {
            self.player_pmove(i);
            self.player_pstep(i);
            return;
        }
        self.cam_shake(1); // move.b #1,$755C
        let p = &mut self.players[i];
        p.b.vy = 0;
        p.b.ay = 0;
        p.b.live = 1;
        self.player_landing_entry(i); // bra 0x1A81A
    }

    /// 0x1BBA6 state 0x32: the hero waits in the vehicle (the car of Episodes 2 and 4) until G.$505 is set, then jumps out: no shadow flag, the pad
    /// 4 (down), no direction (`+$22` = 0xFF), facing right, and the jump entry 0x1A5D0.
    pub(super) fn player_vehicle_wait(&mut self, i: usize) {
        if self.spawn.g.b505 == 0 {
            return;
        }
        let p = &mut self.players[i];
        p.b.draw_flags = 0;
        p.held = DOWN;
        p.b.dir = 0xFF;
        p.b.facing = 1;
        self.player_jump_entry(i);
    }

    /// 0x1BBCA state 0x34 entry (action 0x32 and the stand ins of the ending): the scripted jump to the point (`+$88`, `+$8A`). Up at 0x6D0 with the
    /// acceleration -0x4A (about 47 frames in the air); the speeds in X and Z cover the distance in 47 frames: quotient `d0` and remainder `d1` of
    /// `|distance| / 47` give `vx` = `d1 * 4` + `d0` in the high byte (8.8: pixels per frame and a part of them); the speed in Z is
    /// `d1 * 4 + d0` as a word (the quotient is added to the low byte, as the code does; the distance in Z is 0 in every use). The animation 0x14; in stage 7
    /// area 1 the draw flags 0x30; in stage 2 the held item is dropped.
    pub(crate) fn player_scripted_jump_entry(&mut self, i: usize) {
        self.players[i].b.set_state(0x0200_0434);
        self.player_count_players(); // jsr 0x655E
        let (stage, area) = (self.w.stage, self.cam.s2.area);
        let p = &mut self.players[i];
        p.b.airborne = 1;
        p.b.vy = 0x6D0;
        p.b.ay = -0x4A;
        // X: 0x1BBEA
        let (x, tx) = (p.b.xi() as u16, p.w88);
        let (q, r) = divide_abs(x.wrapping_sub(tx), SCRIPTED_JUMP_FRAMES);
        let (mut d0, mut d1) = (q as u16, r as u16);
        if x >= tx {
            d0 = d0.wrapping_neg(); // bcs skips the neg.w for a target to the right
            d1 = d1.wrapping_neg();
        }
        let vx = d1.wrapping_shl(2); // lsl.w #2,d1
        p.b.vx = vx.wrapping_add((d0 as u8 as u16) << 8) as i16; // move.w d1,$14 / add.b d0,$14 (the high byte)
        p.b.ax = 0;
        // Z: 0x1BC1A
        let (z, tz) = (p.b.zi() as u16, p.w8a);
        let (q, r) = divide_abs(z.wrapping_sub(tz), SCRIPTED_JUMP_FRAMES);
        let (mut d0, mut d1) = (q as u16, r as u16);
        if z >= tz {
            d0 = d0.wrapping_neg();
            if d0 == 0 && (d1 as i16) < 0 {
                d0 = 0xFFFF; // the never taken fix up of the original
            }
            d1 = d1.wrapping_neg();
        }
        p.b.vz = d1.wrapping_shl(2).wrapping_add(d0) as i16;
        p.b.az = 0;
        self.player_panim(i, 0, 0x14);
        if stage == 7 && area == 1 {
            self.players[i].b.draw_flags = 0x30;
        }
        if stage == 2 {
            self.players[i].item_type = 0; // clr.w $B6
        }
    }

    /// 0x1BC8A state 0x34: the jump to the point. The fall speed is at most -0x800. Stages other than 7: moves and animates in the air, lands with
    /// the landing state (0x1A81A) and `live` = 1, all speeds 0. Stage 7 areas 0 and 1 (the lift): when it falls below the height 0x20 it stops there and
    /// the vertical speed and acceleration become the depth speed and acceleration (the lift is seen from the side), X and Y speed 0; stage 7 areas
    /// 2 and 3: landing below 0x20. (The compare of the stage with 2 at 0x1BCBA is dead: the stage is 7 there.)
    pub(super) fn player_scripted_jump(&mut self, i: usize) {
        let (stage, area) = (self.w.stage, self.cam.s2.area);
        let p = &mut self.players[i];
        if p.b.vy < -0x800 {
            p.b.vy = -0x800;
            p.b.ay = 0;
        }
        if stage == 7 && area <= 1 {
            // 0x1BCF0
            if p.b.vy < 0 && p.b.yi() < 0x20 {
                p.b.y = (0x20i32) << 16 | (p.b.y & 0xFFFF);
                p.b.vz = p.b.vy;
                p.b.az = p.b.ay;
                p.b.vx = 0;
                p.b.vy = 0;
                p.b.ay = 0;
            }
        } else if stage == 7 {
            // 0x1BD2E
            if p.b.vy < 0 && p.b.yi() < 0x20 {
                p.b.live = 1;
                return self.player_landing_entry(i);
            }
        } else if p.b.airborne == 0 {
            // 0x1BCC4: on the floor
            p.b.vy = 0;
            p.b.ay = 0;
            p.b.vz = 0;
            p.b.live = 1;
            return self.player_landing_entry(i);
        }
        self.player_pmove(i);
        self.player_pstep(i);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;
    use std::path::Path;
    use std::sync::OnceLock;

    /// The asset pack of the user's ROM (`../../../dino.zip`), `None` when it is not there (the tests that need it skip).
    fn rom() -> Option<&'static AssetPack> {
        static A: OnceLock<Option<&'static AssetPack>> = OnceLock::new();
        *A.get_or_init(|| {
            let path = Path::new("../../../dino.zip");
            path.exists().then(|| &*Box::leak(Box::new(dino_extract::extract(path).expect("the ROM set loads"))))
        })
    }

    /// A game in `stage` and `area` with the heroes `who` (Jack, Mustapha, Hannah) standing in the ground state (0x0200), full HP, none of them
    /// the leader. The floor of the area is loaded (0x12BE4), so the hero state machine can run (`run_frames`).
    fn game(stage: u8, area: u8, who: &[usize]) -> Option<Game> {
        let mut g = Game::new(rom()?);
        g.w.stage = stage;
        g.cam.s2.area = area;
        g.hook_12be4();
        for &k in who {
            g.player_join(k, k as u8);
            g.players[k].b.set_state(0x0200_0000);
            g.player_idle_entry(k);
            g.flow.chosen |= 1 << k;
        }
        g.flow_count_players();
        Some(g)
    }

    /// Runs the script `id` for the heroes of `g` (G.$4DD = all chosen) with the player update's hook only (the heroes do not move or pose), in the
    /// order of the players, and returns the number of frames until G.$4DD is 0 (the frame that reads the END record included), or `None` after
    /// `limit` frames.
    fn run(g: &mut Game, id: u8, limit: u32) -> Option<u32> {
        g.spawn.g.b4de = id;
        g.w.scripted = g.flow.chosen;
        for p in g.players.iter_mut() {
            p.bd0 = 0;
        }
        for frame in 1..=limit {
            for k in 0..3 {
                if g.players[k].b.live != 0 {
                    g.script_player_hook(k);
                }
            }
            if g.w.scripted == 0 {
                return Some(frame);
            }
        }
        None
    }

    /// Like `run`, but with the real player update (`players_update`: pad, script, hero state machine), the clear task's clearing of the pose lock
    /// G.$503 at G.$4E3 and no input. The frame counter G.$0 runs (some states read its parity).
    fn run_frames(g: &mut Game, id: u8, limit: u32) -> Option<u32> {
        g.spawn.g.b4de = id;
        g.w.scripted = g.flow.chosen;
        for p in g.players.iter_mut() {
            p.bd0 = 0;
        }
        for frame in 1..=limit {
            g.w.frame = g.w.frame.wrapping_add(1);
            g.players_update();
            if g.flow.b4e3 != 0 {
                g.flow.script.b503 = 0; // the clear task (0xC4C8) does this when G.$4E3 starts it
            }
            if g.w.scripted == 0 {
                return Some(frame);
            }
        }
        None
    }

    /// D4c A16: the scripts that only wait, hold the pad or walk with a timeout reach their END in the sum of their record timers. Script 9
    /// (the walk right of the starts of Episodes 2, 4 and 7: 4D8, hold right 60, wait 30, END) takes 91 frames and reads its END in the
    /// 92nd (D4c 5.5: 91 to 93), the script 25 (END only) reads it in the first.
    #[test]
    fn a_script_reaches_its_end_in_the_frames_of_its_records() {
        let Some(mut g) = game(1, 2, &[0]) else { return };
        assert_eq!(run(&mut g, 9, 500), Some(92));
        assert_eq!((g.w.scripted, g.players[0].bd0), (0, 0), "the end frees the hero");
        assert_eq!(g.flow.script.b4d8, 0, "and clears G.$4D8");
        let Some(mut g) = game(6, 1, &[0]) else { return };
        assert_eq!(run(&mut g, 25, 500), Some(1));
        // script 10 (unused): 4D8, wait 360, END
        assert_eq!(run(&mut g, 10, 500), Some(362));
    }

    /// The hold right of script 9 is the pad 1 (held and pressed) for 60 frames, G.$4D8 is set from the first frame, G.$506 is set by the
    /// records (1 for the first two, 0 for the wait) and the pad is clear in the wait.
    #[test]
    fn script_9_holds_right_for_60_frames_with_the_clamp_off() {
        let Some(mut g) = game(1, 2, &[0]) else { return };
        g.spawn.g.b4de = 9;
        g.w.scripted = 1;
        let mut pads = Vec::new();
        for _ in 0..91 {
            g.script_player_hook(0);
            pads.push((g.players[0].held, g.players[0].pressed, g.w.no_clamp, g.flow.script.b4d8));
        }
        assert_eq!(pads[0], (0, 0, 1, 1), "4D8 in the first frame");
        assert!(pads[1..61].iter().all(|&p| p == (RIGHT, RIGHT, 1, 1)), "hold right 60 frames");
        assert!(pads[61..].iter().all(|&p| p == (0, 0, 0, 1)), "wait 30 frames, G.$506 off");
    }

    /// Every script of the table ends: with the flags and floors in the state "nothing is going on" (no items, the map card and the boss intro
    /// and the car are over), each of the 26 reaches its END within twice the sum of its record timers plus the 90 frames the item wait adds, with
    /// one, two and three heroes, with the real player update (the walks, the poses, the jumps and the hops are real, the clear task clears the pose
    /// lock). The jump to a point of Episode 7 and 8 needs its stage, so every script runs in the stage that uses it.
    #[test]
    fn all_26_scripts_end() {
        for who in [&[0usize][..], &[0, 2], &[0, 1, 2]] {
            for id in 0..26u8 {
                // the stage and area of the script (D4c 5.5: "used by"); the walks use their tables
                let (stage, area) = match id {
                    1 | 2 | 4 | 5 | 6 | 12 => (0, [0, 0, 0, 0, 1, 1, 2, 0, 0, 0, 0, 0, 2][id as usize]),
                    16 => (6, 0),
                    17 | 18 => (5, 1),
                    19 | 20 => (6, 2),
                    21 | 22 | 23 | 24 => (7, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3][id as usize]),
                    7 | 13 | 15 => (1, 0),
                    14 => (3, 0),
                    _ => (2, 0),
                };
                let Some(mut g) = game(stage, area, who) else { return };
                let limit: u32 = g.assets.script.script(id).records.iter().map(|r| r.timer as u32 + 90).sum::<u32>() * 2 + 100;
                // a leader as the wave end sets it
                g.prop_set_leader();
                if id == 24 {
                    // the clear sequence (0x42) waits for the last boss's `+$A6` (G.$4E0)
                    let boss = g.enemies.alloc().map(|e| g.enemies.handle(e)).unwrap();
                    g.body_mut(boss).live = 1;
                    g.spawn.g.killer = boss;
                    let e = g.enemy_index(boss);
                    g.enemies.slots[e].m.sb(0xA6, 1);
                }
                assert!(run_frames(&mut g, id, limit).is_some(), "script {id} with {} heroes does not end in {limit} frames", who.len());
            }
        }
    }

    /// The end of a script with the action 0x0A frees every hero, with 0x26 (script 12) only the hero himself: the `+$D0` of all heroes
    /// is cleared, so a hero whose bit is still set starts his script again from the first record (D4c 5.5 says "keeps G.$4DD"). The two
    /// heroes are one frame apart after the barrier (the second reads it in the frame of the first), so P2 reaches the end first.
    #[test]
    fn the_action_26_frees_one_hero_and_restarts_the_others() {
        let Some(mut g) = game(0, 2, &[0, 1]) else { return };
        let first = g.assets.script.script(12).records[0];
        g.spawn.g.b4de = 12;
        g.w.scripted = 3;
        let mut step = |g: &mut Game| {
            g.w.frame = g.w.frame.wrapping_add(1);
            g.players_update();
            if g.flow.b4e3 != 0 {
                g.flow.script.b503 = 0; // the clear task
            }
        };
        let mut frames = 0;
        while g.w.scripted == 3 && frames < 2000 {
            step(&mut g);
            frames += 1;
        }
        assert!(frames < 2000, "a hero reads the end");
        // the hero who read it is out of G.$4DD, the other is not; the copies of G.$4DD are cleared
        let (done, other) = if g.w.scripted == 1 { (1, 0) } else { (0, 1) };
        assert_eq!(g.players[done].sc.act, 0x26);
        assert_eq!(g.w.scripted, 1 << other);
        // in the next frame the other hero copies G.$4DD again and reads the first record (the item wait 0x38: 160 frames, no item is around)
        step(&mut g);
        assert_eq!(g.players[other].bd0, 1 << other);
        assert_eq!((g.players[other].sc.act, first.action), (0x38, 0x38));
        assert_eq!(g.players[other].sc.timer, 160);
        // and runs the whole script alone, to the end of it: G.$4DD is 0 then
        let mut more = 0;
        while g.w.scripted != 0 && more < 3000 {
            step(&mut g);
            more += 1;
        }
        assert_eq!(g.w.scripted, 0);
    }

    /// The walk direction (0x1DF10): recomputed in the first frame and then every fifth frame, kept in between; right when the target is not
    /// left of the hero (unsigned words: a hero at a negative X walks left), none inside 4 pixels, towards the viewer (4) when the target Z is
    /// less and away (8) when it is more; the dead zone of Z is the same. The first frames of a walk with the target 0x100 pixels right and 0x40
    /// away: right and away, held and pressed in the first frame, nothing new while it goes on.
    #[test]
    fn the_walk_keeps_its_direction_and_recomputes_it_every_fifth_frame() {
        let Some(mut g) = game(0, 0, &[0]) else { return };
        g.players[0].b.set_xi(0x100);
        g.players[0].b.set_zi(0x80);
        g.spawn.g.b4de = 0; // only the walk body is run below
        let s = &mut g.players[0].sc;
        (s.tx, s.tz, s.count) = (0x200, 0xC0, 0);
        let mut seen = Vec::new();
        for k in 0..7 {
            if k == 2 {
                g.players[0].prev = g.players[0].held; // what the player update does at 0x18968
            }
            g.script_walk_body(0);
            seen.push((g.players[0].held, g.players[0].pressed, g.players[0].sc.count));
            g.players[0].prev = g.players[0].held;
        }
        assert_eq!(seen[0], (RIGHT | UP, RIGHT | UP, 4), "the first frame recomputes");
        assert_eq!(seen[1], (RIGHT | UP, 0, 3), "then the direction stays and nothing is new");
        assert_eq!([seen[4].2, seen[5].2], [0, 4], "every fifth frame: counter 4 3 2 1 0 and again 4");
        // the target left and nearer: left and towards the viewer
        let s = &mut g.players[0].sc;
        (s.tx, s.tz, s.count) = (0x80, 0x40, 0);
        g.script_walk_body(0);
        assert_eq!(g.players[0].held, LEFT | DOWN);
        // inside the dead zone of 4 pixels (X - 4 to X + 3, Z the same): no direction
        let s = &mut g.players[0].sc;
        (s.tx, s.tz, s.count) = (0x100 + 3, 0x80 - 4, 0);
        g.script_walk_body(0);
        assert_eq!(g.players[0].held, 0);
        // 68000 compares: a hero at X -48 (0xFFD0) whose target is 0x100 sees the target "left" of him (unsigned) and walks left
        g.players[0].b.set_xi(-48);
        let s = &mut g.players[0].sc;
        (s.tx, s.tz, s.count) = (0x100, 0x80, 0);
        g.script_walk_body(0);
        assert_eq!(g.players[0].held & (LEFT | RIGHT), LEFT);
    }

    /// The arrival test is made in the frames that recompute the direction: a box of 0xC x 0x18 (0x10 x 0x20 in stage 2) round the target.
    /// A hero who arrived presses nothing, turns right and sets his bit in G.$4DF; when it equals G.$4DD the timers of all heroes are 1.
    #[test]
    fn a_hero_at_his_spot_stops_and_the_timers_go_to_1_when_all_are_there() {
        let Some(mut g) = game(0, 0, &[0, 1]) else { return };
        g.w.scripted = 3;
        for k in 0..2 {
            g.players[k].sc.timer = 300;
            g.players[k].sc.tx = 0x400;
            g.players[k].sc.tz = 0x100;
            g.players[k].b.set_xi(0x400 + 11); // 0xB inside: arrived (X - target + 0xC = 0x17 < 0x18)
            g.players[k].b.set_zi(0x100 - 12);
        }
        g.script_walk_body(0);
        assert_eq!((g.players[0].held, g.players[0].b.facing, g.flow.script.b4df), (0, 1, 1));
        assert_eq!(g.players[1].sc.timer, 300, "P2 is not counted yet");
        g.script_walk_body(1);
        assert_eq!(g.flow.script.b4df, 3);
        assert!(g.players.iter().all(|p| p.sc.timer == 1), "all arrived: the records end");
        // one more pixel and he is outside
        let mut g2 = game(0, 0, &[0]).unwrap();
        g2.w.scripted = 1;
        g2.players[0].sc.tx = 0x400;
        g2.players[0].sc.tz = 0x100;
        g2.players[0].b.set_xi(0x400 + 12);
        g2.players[0].b.set_zi(0x100);
        g2.script_walk_body(0);
        assert_eq!(g2.flow.script.b4df, 0);
        // stage 2 has a wider box
        let mut g3 = game(2, 0, &[0]).unwrap();
        g3.w.scripted = 1;
        g3.players[0].sc.tx = 0x400;
        g3.players[0].sc.tz = 0x100;
        g3.players[0].b.set_xi(0x400 + 12);
        g3.players[0].b.set_zi(0x100);
        g3.script_walk_body(0);
        assert_eq!(g3.flow.script.b4df, 1);
    }

    /// The end targets (0x1DFCA): the leader takes the front spot (slot 0); the others take the free spot that is nearest to them; a taken spot
    /// has the distance 0xFFFF. With both other spots taken a hero gets spot 0 as well (the quirk of the compares).
    #[test]
    fn the_end_walk_gives_the_leader_the_front_spot_and_the_others_the_nearest() {
        let Some(mut g) = game(0, 0, &[0, 1, 2]) else { return };
        let spot = |g: &Game, k: u32| (g.assets.script.tables.u16(tab::END_TARGETS, 4 * k), g.assets.script.tables.u16(tab::END_TARGETS, 4 * k + 2));
        let (s0, s1, s2) = (spot(&g, 0), spot(&g, 1), spot(&g, 2));
        // P2 is the leader; P1 stands next to the spot 2, P3 next to the spot 1
        g.players[1].bc0 = 1;
        g.players[0].b.set_xi(s2.0 as i16 + 3);
        g.players[0].b.set_zi(s2.1 as i16);
        g.players[2].b.set_xi(s1.0 as i16 - 5);
        g.players[2].b.set_zi(s1.1 as i16 + 2);
        g.players[1].b.set_xi(0);
        for k in 0..3 {
            g.script_targets_end(k);
        }
        let got = |g: &Game, k: usize| ((g.players[k].sc.tx, g.players[k].sc.tz), g.players[k].sc.rank);
        assert_eq!(got(&g, 1), (s0, 0), "the leader");
        assert_eq!(got(&g, 0), (s2, 2), "P1 takes the nearest free spot");
        assert_eq!(got(&g, 2), (s1, 1));
        assert_eq!(g.flow.script.taken, [1, 1, 1]);
        // both spots taken: the third hero gets spot 0 (rank 0)
        g.players[2].bc0 = 0;
        g.script_targets_end(2);
        assert_eq!(got(&g, 2), (s0, 0));
        // the two player cabinet reads the row 4 bytes on
        let mut g = game(0, 0, &[0]).unwrap();
        g.players[0].bc0 = 1;
        g.flow.b4cf = 1;
        g.script_targets_end(0);
        assert_eq!((g.players[0].sc.tx, g.players[0].sc.tz), (g.assets.script.tables.u16(tab::END_TARGETS, 4), g.assets.script.tables.u16(tab::END_TARGETS, 6)));
    }

    /// The start targets (0x1E07E, by stage and area through the pointer table) and the single point (0x1E3BE: all heroes get the same).
    #[test]
    fn the_start_targets_and_the_point_come_from_the_tables() {
        let Some(mut g) = game(0, 0, &[0, 1]) else { return };
        let t = &g.assets.script.tables;
        let row = t.u32(tab::START_TARGETS, 0);
        let (x0, z0) = (t.u16(row, 0), t.u16(row, 2));
        let (px, pz) = (t.u16(tab::POINTS, 0), t.u16(tab::POINTS, 2));
        g.players[0].bc0 = 1;
        g.script_targets_start(0);
        assert_eq!((g.players[0].sc.tx, g.players[0].sc.tz), (x0, z0));
        // the single point goes to every hero, present or not
        g.players[0].sc.step = 0;
        g.script_walk_point(0);
        assert!(g.players.iter().all(|p| (p.sc.tx, p.sc.tz) == (px, pz)));
        assert_eq!((g.players[0].sc.step, g.flow.script.b4df), (2, 0));
        // area 2: the third row of the stage
        g.cam.s2.area = 2;
        let (px2, pz2) = (t.u16(tab::POINTS, 8), t.u16(tab::POINTS, 10));
        g.players[1].sc.step = 0;
        g.script_walk_point(1);
        assert_eq!((g.players[1].sc.tx, g.players[1].sc.tz), (px2, pz2));
    }

    /// The lineup (0x1E170, 0x1E13A): the present heroes sorted by X, the right most first, take the three lineup spots in order; an absent hero is skipped.
    #[test]
    fn the_lineup_sorts_the_heroes_by_x() {
        let Some(mut g) = game(0, 2, &[0, 1, 2]) else { return };
        for (k, x) in [(0usize, 100i16), (1, 300), (2, 200)] {
            g.players[k].b.set_xi(x);
        }
        assert_eq!(g.script_sorted_heroes(), vec![1, 2, 0]);
        g.script_targets_lineup();
        let spot = |k: u32| (g.assets.script.tables.u16(tab::LINEUP, 4 * k), g.assets.script.tables.u16(tab::LINEUP, 4 * k + 2));
        assert_eq!((g.players[1].sc.tx, g.players[1].sc.tz), spot(0));
        assert_eq!((g.players[2].sc.tx, g.players[2].sc.tz), spot(1));
        assert_eq!((g.players[0].sc.tx, g.players[0].sc.tz), spot(2));
        // ties and negative X (signed compares): P1 and P2 at the same X keep their order
        g.players[0].b.set_xi(-48);
        g.players[1].b.set_xi(-48);
        g.players[2].b.set_xi(-64);
        assert_eq!(g.script_sorted_heroes(), vec![0, 1, 2]);
        // P2 absent: P1 and P3 take the first two spots
        g.players[1].b.live = 0;
        g.players[0].b.set_xi(10);
        g.players[2].b.set_xi(500);
        assert_eq!(g.script_sorted_heroes(), vec![2, 0]);
    }

    /// The barrier (0x1DC60) waits until every scripted hero is at it with HP and not invulnerable; a free hero object takes his bit out of G.$4DD.
    #[test]
    fn the_barrier_waits_for_the_heroes() {
        let Some(mut g) = game(0, 0, &[0, 1]) else { return };
        g.w.scripted = 3;
        for p in g.players.iter_mut() {
            p.sc.act = 0x4A;
        }
        g.script_barrier(0);
        assert_eq!(g.players[0].sc.timer, 1, "both at the barrier");
        g.players[1].b.live = 2; // invulnerable after a hit or a respawn
        g.script_barrier(0);
        assert_eq!(g.players[0].sc.timer, 0xFFFF);
        g.players[1].b.live = 1;
        g.players[1].b.hp = 0;
        g.script_barrier(0);
        assert_eq!(g.players[0].sc.timer, 0xFFFF, "dead");
        g.players[1].b.hp = 100;
        g.players[1].sc.act = 0x00;
        g.script_barrier(0);
        assert_eq!(g.players[0].sc.timer, 0xFFFF, "not there yet");
        // a free object: his bit goes, and this frame still waits (his action is not the barrier)
        g.players[1].b.live = 0;
        g.script_barrier(0);
        assert_eq!((g.players[0].sc.timer, g.w.scripted), (0xFFFF, 1));
        g.script_barrier(0);
        assert_eq!(g.players[0].sc.timer, 1, "alone he goes on");
    }

    /// The waits: 0x34 looks at the heroes of G.$4DD, 0x30 at the present heroes, both for the airborne flag; 0x20, 0x2E and 0x4C at a global.
    #[test]
    fn the_waits_look_at_their_flags() {
        let Some(mut g) = game(0, 0, &[0, 1]) else { return };
        g.w.scripted = 1;
        g.players[1].b.airborne = 1;
        g.script_wait_ground_scripted(0);
        assert_eq!(g.players[0].sc.timer, 1, "only P1 is scripted");
        g.script_wait_ground_present(0);
        assert_eq!(g.players[0].sc.timer, 0xFFFF, "P2 is present and in the air");
        g.players[1].b.airborne = 0;
        g.players[0].b.airborne = 1;
        g.script_wait_ground_scripted(0);
        assert_eq!(g.players[0].sc.timer, 0xFFFF);
        // the flags
        let setters: [(u8, fn(&mut Game, u8)); 3] = [
            (0x20, |g, v| g.spawn.g.b4fc = v),
            (0x2E, |g, v| g.flow.b4e2 = v),
            (0x4C, |g, v| g.spawn.g.car_mode = v),
        ];
        for (act, set) in setters {
            g.players[0].sc.act = act;
            set(&mut g, 1);
            g.script_action(0);
            assert_eq!(g.players[0].sc.timer, 0xFFFF, "action {act:#x} waits");
            set(&mut g, 0);
            g.script_action(0);
            assert_eq!(g.players[0].sc.timer, 1, "action {act:#x} goes on");
        }
    }

    /// The floor item waits (0x2A, 0x38): an item of the class 0 (0x2A only), 2, 4, 5 or 9 in the mode 0 to 2 with the sub 0 or 0xC counts.
    /// With one the record's time stays; the first frame without one sets 90 (0x2A) or 160 (0x38) and the test is not made again.
    #[test]
    fn the_item_waits_set_90_or_160_frames_when_the_floor_is_clear() {
        let Some(mut g) = game(0, 0, &[0]) else { return };
        g.players[0].sc.timer = 180;
        g.players[0].sc.act = 0x2A;
        let i = g.items.alloc().unwrap();
        let b = &mut g.items.slots[i].b;
        (b.live, b.k20, b.mode, b.sub) = (1, 0x10, 2, 0); // a container lying around
        g.script_action(0);
        assert_eq!((g.players[0].sc.timer, g.players[0].sc.step), (180, 0), "it stays");
        g.items.slots[i].b.k20 = 0; // class 0: a weapon
        g.script_action(0);
        assert_eq!(g.players[0].sc.timer, 180);
        g.players[0].sc.act = 0x38; // 0x38 does not count class 0
        g.script_action(0);
        assert_eq!((g.players[0].sc.timer, g.players[0].sc.step), (160, 2));
        g.players[0].sc.timer = 180;
        g.players[0].sc.step = 0;
        g.players[0].sc.act = 0x2A;
        g.items.slots[i].b.sub = 4; // being picked up: no longer counts
        g.script_action(0);
        assert_eq!((g.players[0].sc.timer, g.players[0].sc.step), (90, 2));
        g.items.slots[i].b.sub = 0;
        g.script_action(0);
        assert_eq!(g.players[0].sc.timer, 90, "the test is made in the first frame only");
    }

    /// The simple actions: 0x28 sets G.$4D8, 0x2C and 0x36 the camera go flag, 0x36 and 0x12 hold right, 0x18 right and jump, 0x08 the leader's
    /// attack, 0x44 / 0x48 G.$4E2, 0x0A and 0x26 clear G.$4E3.
    #[test]
    fn the_simple_actions_do_what_the_table_says() {
        let Some(mut g) = game(0, 0, &[0, 1]) else { return };
        let mut run = |g: &mut Game, act: u8| {
            g.players[0].sc.act = act;
            g.script_action(0);
        };
        run(&mut g, 0x28);
        assert_eq!(g.flow.script.b4d8, 1);
        run(&mut g, 0x2C);
        assert_eq!(g.cam.s2.go, 1);
        g.cam.s2.go = 0;
        run(&mut g, 0x36);
        assert_eq!((g.cam.s2.go, g.players[0].held, g.players[0].pressed), (1, RIGHT, RIGHT));
        run(&mut g, 0x18);
        assert_eq!((g.players[0].held, g.players[0].pressed), (RIGHT | JMP, RIGHT | JMP));
        g.players[0].bc0 = 1;
        run(&mut g, 0x08);
        assert_eq!((g.players[0].held, g.players[0].pressed), (ATK, ATK));
        g.players[1].bc0 = 0;
        g.players[1].sc.act = 0x08;
        g.script_action(1);
        assert_eq!(g.players[1].held, 0);
        run(&mut g, 0x1C);
        assert_eq!(g.players[0].held, ATK, "0x1C leaves the pad alone");
        run(&mut g, 0x00);
        assert_eq!((g.players[0].held, g.players[1].pressed), (0, 0), "a wait clears the pad of all heroes");
        g.players[1].dtap = 7;
        run(&mut g, 0x00);
        assert_eq!(g.players[1].dtap, 0);
        run(&mut g, 0x48);
        assert_eq!(g.flow.b4e2, 0, "not in a game");
        g.flow.in_game = 1;
        run(&mut g, 0x48);
        assert_eq!(g.flow.b4e2, 1);
        run(&mut g, 0x44);
        assert_eq!(g.flow.b4e2, 0);
        g.flow.b4e3 = 1;
        g.w.scripted = 3;
        run(&mut g, 0x0A);
        assert_eq!((g.flow.b4e3, g.w.scripted, g.flow.script.b4d8), (0, 0, 0));
    }

    /// The actions that make objects or ask for a sound (M2-12): 0x22 the caption banner (pool B class 66) of G.$4FD, 0x24 the tile change
    /// (class 21), 0x40 the door smash (class 11 at X 0x670, Z 0x118) and 0x46 the forced GO (class 20, variant 2, the only one of them without the
    /// pad clear), 0x4E the sound of G.$4FD, 0x50 the music fade out. All of them but 0x46 end with 0x1DEA6.
    #[test]
    fn the_object_actions_make_their_objects() {
        let Some(mut g) = game(0, 0, &[0, 1]) else { return };
        g.flow.script.b4fd = 0x49;
        g.players[1].held = ATK;
        g.players[1].pressed = ATK;
        g.players[0].sc.act = 0x22;
        g.script_action(0);
        let live: Vec<_> = g.props.slots.iter().filter(|p| p.b.live != 0).map(|p| (p.b.k20 / 4, p.b.var26)).collect();
        assert_eq!(live, vec![(66, 0x49)], "the banner of the record's argument");
        assert_eq!(g.players[1].held, 0, "the pad of every hero is cleared");
        for (act, class, var) in [(0x24u8, 21u16, 0u16), (0x40, 11, 0), (0x46, 20, 2)] {
            g.players[1].held = ATK;
            g.players[0].sc.act = act;
            g.script_action(0);
            let p = g.props.slots.iter().filter(|p| p.b.live != 0).last().map(|p| (p.b.k20 / 4, p.b.var26)).unwrap();
            let newest = g.props.slots.iter().filter(|p| p.b.live != 0 && p.b.k20 / 4 == class).count();
            assert_eq!(newest, 1, "action {act:#x} makes one class {class} object");
            let _ = (p, var);
            assert_eq!(g.players[1].held == 0, act != 0x46, "action {act:#x}: pad clear");
        }
        let door = g.props.slots.iter().find(|p| p.b.live != 0 && p.b.k20 / 4 == 11).unwrap();
        assert_eq!((door.b.xi(), door.b.zi()), (0x670, 0x118));
        let hint = g.props.slots.iter().find(|p| p.b.live != 0 && p.b.k20 / 4 == 20).unwrap();
        assert_eq!(hint.b.var26, 2);
        g.w.sounds.clear();
        g.flow.script.b4fd = 0x31;
        g.players[0].sc.act = 0x4E;
        g.script_action(0);
        g.players[0].sc.act = 0x50;
        g.script_action(0);
        assert_eq!(g.w.sounds, vec![0x31, 0xFF07]);
    }

    /// 0x42, the clear sequence of Episode 8 (0x1E650): the first run starts it once (G.-$70AA, G.$516, the sound 0x23, the bits of the present
    /// characters in G.-$70A9); every run sets the timer from the boss's `+$A6` (G.$4E0): 1 when it is not 0, else 0xFFFF. No pad clear.
    #[test]
    fn the_clear_sequence_starts_once_and_waits_for_the_boss() {
        let Some(mut g) = game(7, 3, &[0, 2]) else { return };
        let boss = g.enemies.alloc().map(|e| g.enemies.handle(e)).unwrap();
        g.body_mut(boss).live = 1;
        g.spawn.g.killer = boss;
        g.players[0].held = ATK;
        g.players[0].sc.act = 0x42;
        g.script_action(0);
        assert_eq!((g.flow.script.b70aa, g.flow.script.b516, g.players[0].sc.step), (1, 1, 2));
        assert_eq!(g.flow.script.b70a9, 0b101, "Jack (0) and Hannah (2) are on the stage");
        assert_eq!(g.w.sounds.iter().filter(|&&c| c == 0x23).count(), 1);
        assert_eq!(g.players[0].sc.timer, 0xFFFF, "the boss's +$A6 is 0: wait");
        assert_eq!(g.players[0].held, ATK, "no pad clear");
        // the second hero in the same frame does not start it again
        g.players[2].sc.act = 0x42;
        g.script_action(2);
        assert_eq!(g.w.sounds.iter().filter(|&&c| c == 0x23).count(), 1);
        assert_eq!(g.players[2].sc.timer, 0xFFFF);
        let i = g.enemy_index(boss);
        g.enemies.slots[i].m.sb(0xA6, 1);
        g.script_action(0);
        g.script_action(2);
        assert_eq!((g.players[0].sc.timer, g.players[2].sc.timer), (1, 1));
        // a later hero whose own step is still 0 does not start it either (G.-$70AA is set)
        assert_eq!(g.flow.script.b70a9, 0b101);
    }

    /// 0x3C walks right until X reaches 0xB80 (signed), then it is an arrival.
    #[test]
    fn the_action_3c_walks_right_to_b80() {
        let Some(mut g) = game(0, 0, &[0]) else { return };
        g.w.scripted = 1;
        g.players[0].sc.act = 0x3C;
        g.players[0].b.set_xi(0xB00);
        g.script_action(0);
        assert_eq!((g.players[0].held, g.players[0].pressed, g.players[0].sc.tx), (RIGHT, RIGHT, 0xB80));
        g.players[0].b.set_xi(0xB80);
        g.players[0].sc.timer = 200;
        g.script_action(0);
        assert_eq!((g.players[0].held, g.flow.script.b4df, g.players[0].sc.timer), (0, 1, 1));
    }

    /// The area changes that the walk scripts end must not leave stale walk fields: a hero starts a script with `step` 0 and the end slots free.
    #[test]
    fn a_new_script_start_frees_the_end_slots() {
        let Some(mut g) = game(0, 0, &[0]) else { return };
        g.flow.script.taken = [1, 1, 1];
        g.players[0].sc.step = 2;
        g.spawn.g.b4de = 2;
        g.script_hero_start(0);
        assert_eq!(g.flow.script.taken, [0, 0, 0]);
        let s = g.players[0].sc;
        assert_eq!((s.step, s.timer, s.ptr), (0, 1, g.assets.script.pointers[2]));
    }

    /// One frame of the real player update and the pose lock clearing of the clear task (see `run_frames`).
    fn frame(g: &mut Game) {
        g.w.frame = g.w.frame.wrapping_add(1);
        g.players_update();
        if g.flow.b4e3 != 0 {
            g.flow.script.b503 = 0;
        }
    }

    /// The action 0x3A: the state 0x2A (0x19EA6): the hero moves right at 0xC0 (0.75 pixels a frame) and hops in depth: the acceleration -12 for 13
    /// frames, then 9 frames at rest, and again. The action runs once (`+$D3` = 2).
    #[test]
    fn the_hops_state_cycles_every_22_frames() {
        let Some(mut g) = game(0, 0, &[0]) else { return };
        g.w.scripted = 1;
        g.players[0].sc.act = 0x3A;
        g.script_action(0);
        assert_eq!(g.players[0].b.state(), 0x0200_002A);
        assert_eq!((g.players[0].b.vx, g.players[0].b.az, g.players[0].b90), (0xC0, -12, 2));
        g.script_action(0); // the action does not start it again
        assert_eq!(g.players[0].b90, 2);
        let mut phases = vec![g.players[0].b90];
        let mut speeds = vec![(g.players[0].b.vz, g.players[0].b.az)];
        for _ in 0..44 {
            g.player_hops(0);
            phases.push(g.players[0].b90);
            speeds.push((g.players[0].b.vz, g.players[0].b.az));
        }
        // 13 calls in step 2 (the first one is the entry), 9 in step 4, and round again: the step changes in the call that has `+$80` = 0
        let changes: Vec<usize> = (1..phases.len()).filter(|&k| phases[k] != phases[k - 1]).collect();
        assert_eq!(changes, [12, 21, 34, 43], "{phases:?}");
        assert_eq!(speeds[12], (0, 0), "the rest after step 2");
        assert_eq!(speeds[21], (0, -12), "and the hop again after step 4");
        assert!(speeds[5].0 < 0, "in step 2 the depth speed grows towards the viewer");
        assert_eq!(g.players[0].b.vx, 0xC0);
    }

    /// The action 0x3E (leader only): the state 0x2C (0x19F40): the spark at the chest at once, 41 frames of waiting, then the object (pool B class 35
    /// at the hero's place with his handle in `+$76`) and the animation 0x1F, which ends in idle. A hero who is not the leader does nothing.
    #[test]
    fn the_chest_pose_waits_41_frames_and_makes_its_object() {
        let Some(mut g) = game(0, 0, &[0, 1]) else { return };
        g.w.scripted = 3;
        g.players[1].sc.act = 0x3E;
        g.script_action(1);
        assert_eq!(g.players[1].b.state(), 0x0200_0000, "not the leader");
        g.players[0].bc0 = 1;
        g.players[0].b.set_xi(0x300);
        g.players[0].b.set_yi(0);
        g.players[0].b.set_zi(0x80);
        g.players[0].sc.act = 0x3E;
        g.script_action(0);
        assert_eq!((g.players[0].b.state(), g.players[0].b80()), (0x0200_002C, 0x28));
        let sparks = g.effects.slots.iter().filter(|e| e.b.live != 0).count();
        assert_eq!(sparks, 1, "the spark of the chest");
        let spark = g.effects.slots.iter().find(|e| e.b.live != 0).unwrap();
        assert_eq!((spark.b.xi(), spark.b.yi(), spark.b.zi(), spark.b.var26), (0x300, 0x58, 0x80, 0xD));
        for _ in 0..40 {
            g.player_chest_pose(0);
        }
        assert_eq!(g.players[0].b90, 0, "still waiting after 40 calls (+$80 counts 0x27 .. 0)");
        assert!(g.props.slots.iter().all(|p| p.b.live == 0));
        g.player_chest_pose(0);
        assert_eq!(g.players[0].b90, 2, "the 41st call has +$80 = 0 and goes on");
        let prop = g.props.slots.iter().find(|p| p.b.live != 0).unwrap();
        assert_eq!((prop.b.k20, prop.b.xi(), prop.b.zi(), prop.m.w(0x76)), (35 * 4, 0x300, 0x80, Game::player_handle(0)));
        // the animation 0x1F runs to its end and the hero is idle again
        let mut frames = 0;
        while g.players[0].b.state() != 0x0200_0000 && frames < 500 {
            g.player_chest_pose(0);
            frames += 1;
        }
        assert!(frames < 500);
    }

    /// The scripted jump (action 0x32, state 0x34 entry 0x1BBCA): up at 0x6D0, acceleration -0x4A; the speed covers the distance in 47 frames as the
    /// quotient (high byte) and four times the remainder of `|dx| / 47`, negated for a target on the left. Z stays (the target Z is the Z of the hero).
    #[test]
    fn the_scripted_jump_aims_at_the_point() {
        let Some(mut g) = game(6, 0, &[0]) else { return };
        g.players[0].b.set_xi(0x100);
        g.players[0].b.set_zi(0x80);
        g.players[0].w88 = 0x200; // 0x100 to the right: 256 = 5 * 47 + 21
        g.players[0].w8a = 0x80;
        g.player_scripted_jump_entry(0);
        let b = &g.players[0].b;
        assert_eq!(b.state(), 0x0200_0434);
        assert_eq!((b.vy, b.ay, b.airborne), (0x6D0, -0x4A, 1));
        assert_eq!(b.vx, (5 << 8) + 4 * 21);
        assert_eq!((b.vz, b.ax, b.az), (0, 0, 0));
        // 0x40 to the left: 64 = 47 + 17: -(1 * 256 + 4 * 17)
        g.players[0].w88 = 0x100 - 0x40;
        g.player_scripted_jump_entry(0);
        assert_eq!(g.players[0].b.vx, -(256 + 68));
        // the target is the place itself: no speed
        g.players[0].w88 = 0x100;
        g.player_scripted_jump_entry(0);
        assert_eq!(g.players[0].b.vx, 0);
        // the table 0x1DD88 gives the offset of the action 0x32: 0x100 in Episode 7 area 0
        g.players[0].b.set_state(0x0200_0000);
        g.players[0].sc.act = 0x32;
        g.players[0].sc.step = 0;
        g.script_action(0);
        assert_eq!((g.players[0].w88, g.players[0].w8a, g.players[0].sc.step), (0x100 + 0x100, 0x80, 2));
        g.script_action(0); // the second run only clears the pad
        assert_eq!(g.players[0].b.state(), 0x0200_0434);
    }

    /// The scripted jump with the real update: the hero lands about 47 frames later at the target, in the landing state, and walks the script on. Episode
    /// 7 area 0 (the lift): the jump is 0x100 pixels to the right.
    #[test]
    fn the_scripted_jump_lands_at_the_target() {
        let Some(mut g) = game(6, 0, &[0]) else { return };
        g.players[0].b.set_xi(0xA00);
        g.players[0].b.set_zi(0x40);
        g.w.no_clamp = 1; // the records of the script turn the screen clamp off (G.$506)
        g.w.scripted = 1;
        g.players[0].sc.act = 0x32;
        g.players[0].sc.step = 0;
        g.players[0].bd0 = 1;
        g.script_action(0);
        let mut landed = None;
        for k in 1..120 {
            frame(&mut g);
            if g.players[0].b.act != 4 || g.players[0].b.step != 0x34 {
                landed = Some(k);
                break;
            }
        }
        let k = landed.expect("the hero lands");
        assert!((44..=50).contains(&k), "the jump takes 47 frames, landed in frame {k}");
        let x = g.players[0].b.xi();
        assert!((0xA00 + 0xF0..=0xA00 + 0x110).contains(&x), "x = {x:#x}");
        assert_eq!(g.players[0].b.live, 1);
    }

    /// The fall into Episode 1 area 2 (action 0x1A, state 0x1C): the hero starts at the height 544 with the speed 0x500 to the right; no shadow while he is
    /// above 0x100; on the floor the screen shakes (strength 1), `live` = 1 and the landing state follows; the script (6) goes on from there and
    /// ends. The leader takes the animation 0x15.
    #[test]
    fn the_fall_into_episode_1_area_2() {
        let Some(mut g) = game(0, 2, &[0]) else { return };
        assert_eq!(g.players[0].b.yi(), 544, "the start position of the area");
        g.prop_set_leader();
        g.w.scripted = 1;
        g.spawn.g.b4de = 6;
        let (mut shake, mut ended) = (false, None);
        let mut states = Vec::new();
        for k in 0..400 {
            frame(&mut g);
            shake |= g.cam.shake_req == 1;
            let st = g.players[0].b.state();
            if states.last() != Some(&st) {
                states.push(st);
            }
            if g.w.scripted == 0 && ended.is_none() {
                ended = Some(k);
            }
        }
        // the state word of the fall is 0x0200 too: the script runs while he falls (4D8, the fall, 120 frames of nothing, one wait) and ends before he lands
        assert!(ended.is_some_and(|k| (120..130).contains(&k)), "the script ended in frame {ended:?}");
        assert!(shake, "the landing shakes the screen");
        assert_eq!(&states[..3], &[0x0200_0000, 0x0200_041C, 0x0200_0404], "idle, the fall, the landing: {states:x?}");
        assert_eq!((g.players[0].b.yi(), g.players[0].b.live), (0, 1));
        assert!(g.players[0].b.xi() > 32, "the speed to the right carried him");
    }

    /// The vehicle wait (0x1BBA6): the hero stays in the car (state 0x432) until G.$505 is set, then jumps out facing right.
    #[test]
    fn the_hero_waits_in_the_vehicle_until_the_car_is_ready() {
        let Some(mut g) = game(1, 0, &[0]) else { return };
        g.players[0].b.set_state(0x0200_0432);
        g.players[0].b.airborne = 1;
        for _ in 0..5 {
            g.player_vehicle_wait(0);
        }
        assert_eq!(g.players[0].b.state(), 0x0200_0432);
        g.spawn.g.b505 = 1;
        g.players[0].facing_age = 10; // he has been sitting there: the jump does not turn him (0x1A626)
        g.player_vehicle_wait(0);
        assert_eq!((g.players[0].b.state(), g.players[0].b.facing), (0x0200_0400, 1), "the jump state of 0x1A5D0");
        assert_eq!(g.players[0].b.draw_flags, 0);
    }

    /// The special attack (action 0x0C): sound 2, the hero's special state 0x0E (not Mustapha: his is an air special), no pad clear.
    #[test]
    fn the_special_action_starts_the_special() {
        let Some(mut g) = game(0, 0, &[0]) else { return };
        g.w.scripted = 1;
        g.players[0].sc.act = 0x0C;
        g.players[0].held = ATK;
        g.script_action(0);
        assert!(g.w.sounds.contains(&2), "sound 2");
        assert_eq!(g.players[0].b.state(), 0x0200_000E);
        assert_eq!(g.players[0].held, ATK, "the pad stays");
        assert_eq!(g.players[0].special_budget, 0x14);
    }

    /// The pose (action 0x04): waits while a scripted hero is in the air or dead; then G.$503 = 1, sound 0x18 and the pose state 0x18 for every hero
    /// (the leader with the animation 0xE, the others 0x11); the pose holds while G.$503 is set.
    #[test]
    fn the_pose_waits_for_the_heroes_and_holds_the_pose_lock() {
        let Some(mut g) = game(0, 0, &[0, 1]) else { return };
        g.w.scripted = 3;
        g.prop_set_leader();
        g.players[1].b.airborne = 1;
        for k in 0..2 {
            g.players[k].sc.act = 0x04;
        }
        g.script_action(0);
        assert_eq!((g.players[0].sc.timer, g.flow.script.b503), (0xFFFF, 0), "P2 is in the air");
        g.players[1].b.airborne = 0;
        g.players[1].b.hp = 0;
        g.script_action(0);
        assert_eq!(g.players[0].sc.timer, 0xFFFF, "P2 is dead");
        g.players[1].b.hp = 100;
        g.w.sounds.clear();
        g.script_action(0);
        assert_eq!((g.players[0].sc.timer, g.flow.script.b503), (1, 1));
        assert!(g.w.sounds.contains(&0x18));
        assert_eq!(g.players[0].b.state(), 0x0200_0018);
        g.script_action(1);
        assert_eq!(g.players[1].b.state(), 0x0200_0018);
        let leader_frame = g.players[1].b.frame;
        let other_frame = g.players[0].b.frame;
        assert_ne!(leader_frame, other_frame, "the leader (P2, the right most at X 0) and the other have their own animations (0xE and 0x11)");
        // the pose runs its animation, then holds while G.$503 is set
        for _ in 0..400 {
            g.player_pose(0);
        }
        assert_eq!(g.players[0].b.state(), 0x0200_0018, "held by the pose lock");
        g.flow.script.b503 = 0;
        g.player_pose(0);
        assert_eq!(g.players[0].b.state(), 0x0200_0000, "the lock is off: idle");
    }

    /// The pose of a leader who is Mustapha (id 1) or Hannah (id 2) is an air pose: the states 0x418 and 0x41A (0x1ADA6, 0x1AE62). Jack (0) and Mess (3)
    /// pose on the ground (0x18). Every pose ends when the pose lock G.$503 is cleared, in the idle state; the heroes who are not the leader pose
    /// on the ground with the animation 0x11.
    #[test]
    fn the_leader_poses_in_the_air_when_he_is_mustapha_or_hannah() {
        for (chr, state) in [(0u8, 0x0200_0018u32), (1, 0x0200_0418), (2, 0x0200_041A), (3, 0x0200_0018)] {
            let Some(mut g) = game(2, 0, &[]) else { return };
            g.player_join(0, chr);
            g.players[0].b.set_state(0x0200_0000);
            g.player_idle_entry(0);
            g.flow.chosen = 1;
            g.flow_count_players();
            g.players[0].bc0 = 1;
            g.flow.script.b503 = 1;
            g.player_pose_entry(0);
            assert_eq!(g.players[0].b.state() >> 16, 0x0200);
            let mut seen = std::collections::BTreeSet::new();
            for _ in 0..600 {
                g.w.frame = g.w.frame.wrapping_add(1);
                g.players_update();
                seen.insert(g.players[0].b.state());
            }
            assert!(seen.contains(&state), "character {chr}: pose state {state:#x}, seen {seen:x?}");
            assert_eq!(g.players[0].b.state(), state, "held by the pose lock");
            g.flow.script.b503 = 0;
            for _ in 0..5 {
                g.w.frame = g.w.frame.wrapping_add(1);
                g.players_update();
            }
            assert_eq!(g.players[0].b.state(), 0x0200_0000, "character {chr}: the lock is off, idle again");
            assert_eq!((g.players[0].b.yi(), g.players[0].b.airborne), (0, 0), "on the floor");
        }
    }

    /// The jump kick (action 0x16): the heroes jump right in the order of their ranks, one frame apart; Jack as the leader presses attack 10 frames after
    /// he jumps; another character as the leader does the dash attack state instead.
    #[test]
    fn the_jump_kick_goes_off_in_the_order_of_the_ranks() {
        let Some(mut g) = game(0, 1, &[0, 1, 2]) else { return };
        g.w.scripted = 7;
        for (k, rank) in [0u8, 2, 1].into_iter().enumerate() {
            g.players[k].sc.act = 0x16;
            g.players[k].sc.rank = rank;
            g.players[k].held = RIGHT;
        }
        g.players[0].bc0 = 1;
        let mut jumps = [None; 3];
        let mut attack = None;
        for frame in 0..30 {
            for k in 0..3 {
                g.script_action(k);
                if g.players[k].held == JMP | RIGHT && jumps[k].is_none() {
                    jumps[k] = Some(frame);
                }
                if g.players[k].held == ATK && attack.is_none() {
                    attack = Some(frame);
                }
            }
        }
        // the first run clears the pad and counts, rank 0 jumps in the frame after it, rank 1 one later, rank 2 two later
        assert_eq!(jumps, [Some(1), Some(3), Some(2)]);
        assert_eq!(attack, Some(1 + 9), "Jack presses attack when his counter runs out, 9 frames after the jump frame");
        // another character as the leader makes the dash attack: no jump pad
        let mut g = game(0, 1, &[1]).unwrap();
        g.w.scripted = 2;
        g.players[1].sc.act = 0x16;
        g.players[1].sc.rank = 0;
        g.players[1].bc0 = 1;
        g.script_action(1);
        g.script_action(1);
        assert_eq!(g.players[1].b.state() >> 16, 0x0200, "the lunge state of the dash attack");
        assert_ne!(g.players[1].held, JMP | RIGHT);
    }

    /// 0x18A28: while G.$516 is set (the clear sequence of Episode 8) the hero state machine, the collision list and the drawing are skipped, in the car
    /// (G.$518) the hero is neither listed nor drawn, and his respawn timer runs.
    #[test]
    fn the_special_modes_skip_the_hero_update() {
        let Some(mut g) = game(7, 0, &[0]) else { return };
        g.players[0].held = 0;
        frame(&mut g);
        assert_eq!(g.combat.get(0x6718).len(), 1, "listed");
        g.combat.get(0x6718);
        g.combat = Default::default();
        g.players[0].b.set_state(0x0200_0000);
        let anim = g.players[0].b.frame;
        g.flow.script.b516 = 1;
        let timer = g.players[0].b.timer;
        frame(&mut g);
        assert_eq!(g.players[0].b.timer, timer, "no animation step");
        assert_eq!(g.players[0].b.frame, anim);
        assert!(g.combat.get(0x6718).is_empty(), "not listed");
        g.flow.script.b516 = 0;
        g.spawn.g.car_mode = 1;
        g.players[0].respawn_t = 2;
        g.players[0].b.live = 2;
        frame(&mut g);
        frame(&mut g);
        assert_eq!((g.players[0].respawn_t, g.players[0].b.live), (0, 1), "the timer runs in the car and the hero is alive again");
        assert!(g.combat.get(0x6718).is_empty(), "the car hides the hero");
    }

    /// The entry points and their tables (D4c 5.3): the start ids, the end ids with G.$514 and the boss area ids.
    #[test]
    fn the_entry_points_pick_the_script_by_stage_and_area() {
        let Some(mut g) = game(0, 0, &[0, 1]) else { return };
        // (stage, area) -> start id
        for (stage, area, id) in [(0u8, 0u8, 1u8), (0, 1, 4), (0, 2, 6), (1, 0, 13), (1, 2, 9), (2, 0, 8), (3, 0, 14), (6, 1, 25), (7, 0, 8)] {
            g.w.stage = stage;
            g.cam.s2.area = area;
            g.script_area_start();
            assert_eq!((g.spawn.g.b4de, g.w.scripted), (id, 3), "start script of stage {stage} area {area}");
        }
        // (stage, area) -> end id, G.$514
        for (stage, area, id, keep) in [(0u8, 0u8, 2u8, 0u8), (0, 1, 5, 0), (0, 2, 12, 0), (1, 1, 15, 1), (3, 0, 0, 1), (6, 3, 20, 0), (7, 3, 24, 0)] {
            g.w.stage = stage;
            g.cam.s2.area = area;
            g.script_area_end();
            assert_eq!((g.spawn.g.b4de, g.flow.script.b514), (id, keep), "end script of stage {stage} area {area}");
        }
        // the window script clears the go flag
        g.cam.s2.go = 1;
        (g.w.stage, g.cam.s2.area) = (0, 1);
        g.script_area_end();
        assert_eq!(g.cam.s2.go, 0);
        // the boss area: (stage) -> id, G.$514, and G.$4FC is set
        for (stage, id, keep) in [(0u8, 11u8, 1u8), (1, 11, 1), (2, 0, 1), (7, 23, 1)] {
            g.w.stage = stage;
            g.spawn.g.b4fc = 0;
            g.script_boss_area();
            assert_eq!((g.spawn.g.b4de, g.flow.script.b514, g.spawn.g.b4fc), (id, keep, 1), "boss area of stage {stage}");
        }
        // the area start clears the leader flags, except in Episode 1 area 2
        g.players[1].bc0 = 1;
        (g.w.stage, g.cam.s2.area) = (0, 2);
        g.script_area_start();
        assert_eq!(g.players[1].bc0, 1, "kept in Episode 1 area 2");
        (g.w.stage, g.cam.s2.area) = (0, 1);
        g.script_area_start();
        assert_eq!(g.players[1].bc0, 0);
        // every entry point clears the copies of G.$4DD in the heroes
        g.players[0].bd0 = 3;
        g.script_area_end();
        assert_eq!(g.players[0].bd0, 0);
    }
}
